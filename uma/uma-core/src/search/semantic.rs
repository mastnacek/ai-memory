//! Semantic vector search: cosine similarity against stored embeddings.

use super::{SearchHit, SearchOptions};
use crate::domain::{FactId, FactStatus, FactType, Scope};
use crate::embeddings::{cosine_similarity, EmbeddingClient};
use crate::indexer::parse_scope_str;
use crate::vector_store::load_all_embeddings;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::PathBuf;
use std::str::FromStr;

/// Performs Semantic Vector Search across facts using cosine similarity.
pub fn search_semantic(
    conn: &Connection,
    opts: &SearchOptions,
    client: &EmbeddingClient,
) -> Result<Vec<SearchHit>> {
    let query_vector = client.embed_one(opts.query)?;
    let stored_vectors = load_all_embeddings(conn)?;

    if stored_vectors.is_empty() {
        return Ok(Vec::new());
    }

    let mut sql = String::from(
        "SELECT id, scope, project_name, fact_type, title, description, body, tags, status, supersedes, file_path, since, until, stale_after
         FROM facts_fts WHERE 1=1",
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

    let mut params_vec: Vec<String> = Vec::new();
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
        params_vec.iter().map(|s| s as &dyn rusqlite::ToSql),
    ))?;

    let mut hits = Vec::new();

    while let Some(row) = rows.next()? {
        let id_str: String = row.get(0)?;
        let scope_str: String = row.get(1)?;
        let project_name: String = row.get(2)?;
        let type_str: String = row.get(3)?;
        let title: String = row.get(4)?;
        let description: Option<String> = row.get(5)?;
        let body: String = row.get(6)?;
        let tags_str: String = row.get(7)?;
        let status_str: String = row.get(8).unwrap_or_else(|_| "stable".to_string());
        let supersedes_str: Option<String> = row.get(9).ok();
        let file_path_str: String = row.get(10)?;
        let since_str: String = row.get(11).unwrap_or_default();
        let until_str: Option<String> = row.get(12).ok();
        let stale_str: Option<String> = row.get(13).ok();

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

        if let Ok(id) = FactId::from_str(&id_str) {
            if let Some(stored_vec) = stored_vectors.get(&id) {
                let similarity = cosine_similarity(&query_vector, stored_vec);
                let file_path = if file_path_str.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(file_path_str))
                };

                let snippet = if let Some(ref d) = description {
                    d.clone()
                } else if body.len() > 140 {
                    format!("{}...", &body[..140].replace('\n', " "))
                } else {
                    body.replace('\n', " ")
                };

                let supersedes = supersedes_str.and_then(|s| FactId::from_str(&s).ok());

                hits.push(SearchHit {
                    id,
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
                    score: similarity,
                });
            }
        }
    }

    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits.truncate(opts.limit);
    Ok(hits)
}
