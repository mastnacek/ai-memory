use crate::domain::{FactId, FactStatus, FactType, Scope};
use crate::embeddings::{cosine_similarity, EmbeddingClient};
use crate::indexer::{build_fts_query, build_safe_fts_query, parse_scope_str};
use crate::vector_store::load_all_embeddings;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub id: FactId,
    pub scope: Scope,
    pub project_name: String,
    pub fact_type: FactType,
    pub title: String,
    pub description: Option<String>,
    pub snippet: String,
    pub tags: Vec<String>,
    pub status: FactStatus,
    pub supersedes: Option<FactId>,
    pub file_path: Option<PathBuf>,
    pub score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Keyword,
    Semantic,
    Hybrid,
}

impl std::str::FromStr for SearchMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "keyword" | "bm25" => Ok(SearchMode::Keyword),
            "semantic" | "vector" => Ok(SearchMode::Semantic),
            "hybrid" | "rrf" => Ok(SearchMode::Hybrid),
            other => anyhow::bail!(
                "Unknown search mode '{}'. Use 'keyword', 'semantic', or 'hybrid'.",
                other
            ),
        }
    }
}

pub struct SearchOptions<'a> {
    pub query: &'a str,
    pub scope: Option<&'a Scope>,
    pub current_project: Option<&'a str>,
    pub fact_type: Option<&'a FactType>,
    pub include_deprecated: bool,
    pub as_of: Option<DateTime<Utc>>,
    pub limit: usize,
}

/// Performs BM25-ranked keyword search across all indexed facts using FTS5.
pub fn search_keyword(conn: &Connection, opts: &SearchOptions) -> Result<Vec<SearchHit>> {
    let trimmed_query = opts.query.trim();
    if trimmed_query.is_empty() {
        return Ok(Vec::new());
    }

    let fts_query = build_fts_query(trimmed_query);
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

    let execute_search = |fts_q: &str| -> Result<Vec<SearchHit>> {
        let mut params_vec: Vec<String> = vec![fts_q.to_string()];
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
    };

    match execute_search(&fts_query) {
        Ok(res) => Ok(res),
        Err(_) => execute_search(&build_safe_fts_query(trimmed_query)),
    }
}

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

/// Merges keyword and semantic search hits using Reciprocal Rank Fusion (RRF).
pub fn reciprocal_rank_fusion(
    keyword_hits: Vec<SearchHit>,
    semantic_hits: Vec<SearchHit>,
    limit: usize,
) -> Vec<SearchHit> {
    if semantic_hits.is_empty() {
        let mut res = keyword_hits;
        res.truncate(limit);
        return res;
    }

    let mut rrf_map: HashMap<FactId, (f64, SearchHit)> = HashMap::new();
    let k = 60.0;

    for (rank, hit) in keyword_hits.into_iter().enumerate() {
        let score = 1.0 / (k + (rank as f64) + 1.0);
        rrf_map.insert(hit.id, (score, hit));
    }

    for (rank, hit) in semantic_hits.into_iter().enumerate() {
        let score = 1.0 / (k + (rank as f64) + 1.0);
        if let Some(entry) = rrf_map.get_mut(&hit.id) {
            entry.0 += score;
        } else {
            rrf_map.insert(hit.id, (score, hit));
        }
    }

    let mut final_hits: Vec<SearchHit> = rrf_map
        .into_values()
        .map(|(score, mut hit)| {
            hit.score = score;
            hit
        })
        .collect();

    final_hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    final_hits.truncate(limit);
    final_hits
}

/// Executes Hybrid Search combining BM25 keyword matching with Semantic Vector search.
pub fn search_hybrid(
    conn: &Connection,
    opts: &SearchOptions,
    client: &EmbeddingClient,
) -> Result<Vec<SearchHit>> {
    let keyword_hits = search_keyword(conn, opts)?;
    let semantic_hits = search_semantic(conn, opts, client)?;
    Ok(reciprocal_rank_fusion(
        keyword_hits,
        semantic_hits,
        opts.limit,
    ))
}
