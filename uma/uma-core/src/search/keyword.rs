//! BM25 keyword search over the FTS5 index.
//!
//! The query is tried as a strict FTS5 expression first and retried with a
//! sanitized form when that fails, so a query containing FTS operators (or a
//! stray quote) degrades to something searchable instead of erroring.

use super::{SearchHit, SearchOptions};
use crate::domain::{FactId, FactStatus, FactType, Scope};
use crate::indexer::{build_fts_query, build_fts_query_or, build_safe_fts_query, parse_scope_str};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::PathBuf;
use std::str::FromStr;

/// Performs BM25-ranked keyword search across all indexed facts using FTS5.
///
/// FTS5 joins tokens with an implicit AND, so a multi-token query only hits
/// when one fact contains every term. The any-term relaxation for the
/// recall path lives in [`search_keyword_or`].
pub fn search_keyword(conn: &Connection, opts: &SearchOptions) -> Result<Vec<SearchHit>> {
    let trimmed_query = opts.query.trim();
    if trimmed_query.is_empty() {
        return Ok(Vec::new());
    }
    run_with_fallback(conn, opts, &build_fts_query(trimmed_query))
}

/// Keyword search relaxed to any-term (OR) matching.
///
/// The strict form starves exactly the recall gate's queries: six words
/// demanding all six prefixes in a single fact returned nothing even when
/// the top-ranked fact was the very bug being asked about. Callers must
/// surface that the relaxation happened — a relaxed result is not the same
/// ranking the caller asked for.
pub fn search_keyword_or(conn: &Connection, opts: &SearchOptions) -> Result<Vec<SearchHit>> {
    let trimmed_query = opts.query.trim();
    if trimmed_query.is_empty() {
        return Ok(Vec::new());
    }
    run_with_fallback(conn, opts, &build_fts_query_or(trimmed_query))
}

/// Runs the SELECT with the given FTS expression, falling back to the
/// sanitized (fully quoted) form when the expression itself fails to parse.
fn run_with_fallback(
    conn: &Connection,
    opts: &SearchOptions,
    fts_query: &str,
) -> Result<Vec<SearchHit>> {
    let trimmed_query = opts.query.trim();
    match run_search(conn, opts, fts_query) {
        Ok(res) => Ok(res),
        Err(_) => run_search(conn, opts, &build_safe_fts_query(trimmed_query)),
    }
}

fn run_search(conn: &Connection, opts: &SearchOptions, fts_query: &str) -> Result<Vec<SearchHit>> {
    let mut sql = String::from(
        "SELECT id, scope, project_name, fact_type, title, description,
                snippet(facts_fts, 6, '[match]', '[/match]', '...', 16) AS snippet,
                tags, status, supersedes, file_path, since, until, stale_after,
                bm25(facts_fts, 0.0, 1.0, 1.0, 1.0, 10.0, 5.0, 2.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0) AS rank
         FROM facts_fts WHERE facts_fts MATCH ?1",
    );

    if !opts.include_deprecated && opts.as_of.is_none() {
        sql.push_str(" AND (status = 'stable' OR status IS NULL)");
    }

    match opts.scope {
        Some(Scope::Global) => sql.push_str(" AND scope = 'global'"),
        Some(Scope::Project(_)) => sql.push_str(" AND project_name = ?"),
        None => {
            if opts.current_project.is_some() {
                sql.push_str(" AND (project_name = ? OR scope = 'global')");
            }
        }
    }

    if opts.fact_type.is_some() {
        sql.push_str(" AND fact_type = ?");
    }

    sql.push_str(" ORDER BY rank ASC LIMIT ?");

    let mut params_vec: Vec<String> = vec![fts_query.to_string()];
    match opts.scope {
        Some(Scope::Project(proj)) => params_vec.push(proj.clone()),
        None => {
            if let Some(cp) = opts.current_project {
                params_vec.push(cp.to_string());
            }
        }
        _ => {}
    }
    if let Some(ft) = opts.fact_type {
        params_vec.push(ft.to_string());
    }

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(rusqlite::params_from_iter(
        params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .chain(std::iter::once(&opts.limit as &dyn rusqlite::ToSql)),
    ))?;

    let mut hits = Vec::new();
    while let Some(row) = rows.next()? {
        let id_str: String = row.get(0)?;
        let scope_str: String = row.get(1)?;
        let project_name: String = row.get(2)?;
        let type_str: String = row.get(3)?;
        let title: String = row.get(4)?;
        let description: Option<String> = row.get(5)?;
        let snippet: String = row.get(6)?;
        let tags_str: String = row.get(7)?;
        let status_str: String = row.get(8).unwrap_or_else(|_| "stable".to_string());
        let supersedes_str: Option<String> = row.get(9).ok();
        let file_path_str: String = row.get(10)?;
        let since_str: String = row.get(11).unwrap_or_default();
        let until_str: Option<String> = row.get(12).ok();
        let stale_str: Option<String> = row.get(13).ok();
        let rank: f64 = row.get(14)?;

        let status = FactStatus::from_str(&status_str).unwrap_or(FactStatus::Stable);

        if let Some(target_dt) = opts.as_of {
            if let Ok(since_dt) = DateTime::parse_from_rfc3339(&since_str) {
                if since_dt.with_timezone(&Utc) > target_dt {
                    continue;
                }
            }
            if let Some(ref u_str) = until_str {
                if let Ok(until_dt) = DateTime::parse_from_rfc3339(u_str) {
                    if target_dt >= until_dt.with_timezone(&Utc) {
                        continue;
                    }
                }
            }
            if let Some(ref s_str) = stale_str {
                if let Ok(stale_dt) = DateTime::parse_from_rfc3339(s_str) {
                    if target_dt >= stale_dt.with_timezone(&Utc) {
                        continue;
                    }
                }
            }
        }

        let file_path = if file_path_str.is_empty() {
            None
        } else {
            Some(PathBuf::from(file_path_str))
        };
        let supersedes = supersedes_str.and_then(|s| FactId::from_str(&s).ok());

        hits.push(SearchHit {
            id: FactId::from_str(&id_str)?,
            scope: parse_scope_str(&scope_str),
            project_name,
            fact_type: FactType::from_str(&type_str).unwrap_or(FactType::Note),
            title,
            description,
            snippet,
            tags: tags_str.split_whitespace().map(String::from).collect(),
            status,
            supersedes,
            file_path,
            score: -rank,
        });
    }
    Ok(hits)
}
