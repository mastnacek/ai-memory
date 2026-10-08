use crate::domain::{FactId, FactType, Scope};
use crate::embeddings::{cosine_similarity, EmbeddingClient};
use crate::indexer::{parse_scope_str, Indexer};
use crate::vector_store::load_all_embeddings;
use anyhow::Result;
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
    pub snippet: String,
    pub tags: Vec<String>,
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

/// Performs Semantic Vector Search across facts using cosine similarity.
pub fn search_semantic(
    conn: &Connection,
    query: &str,
    scope: Option<&Scope>,
    current_project: Option<&str>,
    fact_type: Option<&FactType>,
    limit: usize,
    client: &EmbeddingClient,
) -> Result<Vec<SearchHit>> {
    let query_vector = client.embed_one(query)?;
    let stored_vectors = load_all_embeddings(conn)?;

    if stored_vectors.is_empty() {
        return Ok(Vec::new());
    }

    let mut sql = String::from(
        "SELECT id, scope, project_name, fact_type, title, body, tags, file_path
         FROM facts_fts WHERE 1=1",
    );

    match scope {
        Some(Scope::Global) => {
            sql.push_str(" AND scope = 'global'");
        }
        Some(Scope::Project(_)) => {
            sql.push_str(" AND project_name = ?");
        }
        None => {
            if current_project.is_some() {
                sql.push_str(" AND (project_name = ? OR scope = 'global')");
            }
        }
    }

    if fact_type.is_some() {
        sql.push_str(" AND fact_type = ?");
    }

    let mut params_vec: Vec<String> = Vec::new();
    match scope {
        Some(Scope::Project(proj)) => params_vec.push(proj.clone()),
        None => {
            if let Some(cp) = current_project {
                params_vec.push(cp.to_string());
            }
        }
        _ => {}
    }
    if let Some(ft) = fact_type {
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
        let body: String = row.get(5)?;
        let tags_str: String = row.get(6)?;
        let file_path_str: String = row.get(7)?;

        if let Ok(id) = FactId::from_str(&id_str) {
            if let Some(stored_vec) = stored_vectors.get(&id) {
                let similarity = cosine_similarity(&query_vector, stored_vec);
                let file_path = if file_path_str.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(file_path_str))
                };

                let snippet = if body.len() > 140 {
                    format!("{}...", &body[..140].replace('\n', " "))
                } else {
                    body.replace('\n', " ")
                };

                hits.push(SearchHit {
                    id,
                    scope: parse_scope_str(&scope_str),
                    project_name,
                    fact_type: FactType::from_str(&type_str).unwrap_or(FactType::Note),
                    title,
                    snippet,
                    tags: tags_str.split_whitespace().map(String::from).collect(),
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
    hits.truncate(limit);
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
    indexer: &Indexer,
    query: &str,
    scope: Option<&Scope>,
    current_project: Option<&str>,
    fact_type: Option<&FactType>,
    limit: usize,
    client: &EmbeddingClient,
) -> Result<Vec<SearchHit>> {
    let fetch_limit = limit * 2;
    let keyword_hits =
        indexer.search_keyword(query, scope, current_project, fact_type, fetch_limit)?;
    let semantic_hits = search_semantic(
        indexer.connection(),
        query,
        scope,
        current_project,
        fact_type,
        fetch_limit,
        client,
    )?;

    Ok(reciprocal_rank_fusion(keyword_hits, semantic_hits, limit))
}
