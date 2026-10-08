//! Search: BM25 keyword, semantic vectors, and hybrid fusion over one fact set.
//!
//! Three retrieval concerns, deliberately separated:
//! - [`keyword`] — FTS5/BM25 ranking. Cannot fail for environmental reasons.
//! - [`semantic`] — cosine similarity over stored embeddings. Needs a key.
//! - [`fusion`] — Reciprocal Rank Fusion of the two ranked lists.
//!
//! Hybrid search needs an embedding client *and* stored vectors *and* a working
//! embedding call. When any of those is missing it degrades to keyword-only —
//! and **reports that it did**, via [`outcome::SearchOutcome`]. A caller who
//! asked for hybrid ranking and silently received BM25 has been misled rather
//! than merely served differently, which is the same hazard class as a stale
//! index: confidently wrong, and invisible.

mod fusion;
mod keyword;
mod outcome;
mod semantic;

pub use fusion::reciprocal_rank_fusion;
pub use keyword::search_keyword;
pub use outcome::{Degradation, DegradationReason, SearchOutcome};
pub use semantic::search_semantic;

use crate::domain::{FactId, FactStatus, FactType, Scope};
use crate::embeddings::EmbeddingClient;
use crate::vector_store::count_embeddings;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::PathBuf;

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

/// Executes Hybrid Search combining BM25 keyword matching with Semantic Vector search.
///
/// Both halves must succeed; for the resilient variant used by callers that
/// cannot know in advance whether embeddings are available, see
/// [`search_hybrid_resilient`].
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

/// Hybrid search that falls back to keyword-only, and reports that it did.
///
/// The degradation is part of the return value rather than a log line, so it
/// cannot be dropped on the way to the user.
pub fn search_hybrid_resilient(conn: &Connection, opts: &SearchOptions) -> Result<SearchOutcome> {
    hybrid_with_client_factory(conn, opts, || EmbeddingClient::new(None))
}

/// The degradation logic, with client construction injected so every fallback
/// path is testable without an API key, a network, or a live endpoint.
fn hybrid_with_client_factory<F>(
    conn: &Connection,
    opts: &SearchOptions,
    make_client: F,
) -> Result<SearchOutcome>
where
    F: FnOnce() -> Result<EmbeddingClient>,
{
    // With nothing vectorized the semantic half cannot contribute, so skip
    // embedding the query at all — that would be a wasted API call.
    if count_embeddings(conn).unwrap_or(0) == 0 {
        return degrade_to_keyword(conn, opts, DegradationReason::NoStoredEmbeddings);
    }

    let client = match make_client() {
        Ok(client) => client,
        Err(err) => {
            return degrade_to_keyword(
                conn,
                opts,
                DegradationReason::EmbeddingsUnavailable(err.to_string()),
            )
        }
    };

    // BM25 is computed first and kept regardless: it is the half that cannot
    // fail, so a semantic error must never discard it.
    let keyword_hits = search_keyword(conn, opts)?;
    let semantic = search_semantic(conn, opts, &client);
    Ok(combine(keyword_hits, semantic, opts.limit))
}

/// Pure combination step, so degradation is testable with no I/O at all.
fn combine(
    keyword_hits: Vec<SearchHit>,
    semantic: Result<Vec<SearchHit>>,
    limit: usize,
) -> SearchOutcome {
    match semantic {
        Ok(semantic_hits) => {
            SearchOutcome::complete(reciprocal_rank_fusion(keyword_hits, semantic_hits, limit))
        }
        Err(err) => SearchOutcome {
            hits: keyword_hits,
            degraded: Some(Degradation {
                requested: SearchMode::Hybrid,
                performed: SearchMode::Keyword,
                reason: DegradationReason::SemanticSearchFailed(err.to_string()),
            }),
        },
    }
}

fn degrade_to_keyword(
    conn: &Connection,
    opts: &SearchOptions,
    reason: DegradationReason,
) -> Result<SearchOutcome> {
    Ok(SearchOutcome {
        hits: search_keyword(conn, opts)?,
        degraded: Some(Degradation {
            requested: SearchMode::Hybrid,
            performed: SearchMode::Keyword,
            reason,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Fact;
    use crate::indexer::Indexer;
    use crate::vector_store::save_embedding;
    use tempfile::tempdir;

    fn fact(title: &str, body: &str) -> Fact {
        Fact::new(
            Scope::Global,
            FactType::Decision,
            title.to_string(),
            body.to_string(),
        )
    }

    fn options(query: &str) -> SearchOptions<'_> {
        SearchOptions {
            query,
            scope: None,
            current_project: None,
            fact_type: None,
            include_deprecated: false,
            as_of: None,
            limit: 10,
        }
    }

    #[test]
    fn test_hybrid_without_vectors_degrades_and_still_returns_hits() -> Result<()> {
        let dir = tempdir()?;
        let indexer = Indexer::open(dir.path().join("index.db"))?;
        indexer.index_fact(
            &fact("Adopt SQLite for FTS5 Indexing", "Embedded SQLite FTS5."),
            None,
        )?;

        let outcome = search_hybrid_resilient(indexer.connection(), &options("FTS5"))?;

        assert!(outcome.is_degraded(), "the fallback must be visible");
        let degradation = outcome.degraded.as_ref().expect("degradation");
        assert_eq!(degradation.reason, DegradationReason::NoStoredEmbeddings);
        assert_eq!(degradation.performed, SearchMode::Keyword);
        // Degrading must not mean losing results.
        assert_eq!(outcome.hits.len(), 1);
        assert_eq!(outcome.hits[0].title, "Adopt SQLite for FTS5 Indexing");
        assert!(outcome.note().is_some());
        Ok(())
    }

    #[test]
    fn test_unbuildable_client_degrades_rather_than_erroring() -> Result<()> {
        let dir = tempdir()?;
        let indexer = Indexer::open(dir.path().join("index.db"))?;
        indexer.index_fact(&fact("Adopt SQLite for FTS5 Indexing", "FTS5."), None)?;
        // Store a vector so the "nothing vectorized" branch is not taken and the
        // client really is requested.
        save_embedding(
            indexer.connection(),
            &FactId::new(),
            "test-model",
            &[0.1, 0.2, 0.3],
        )?;

        let outcome = hybrid_with_client_factory(indexer.connection(), &options("FTS5"), || {
            anyhow::bail!("OPENROUTER_API_KEY is not set")
        })?;

        assert_eq!(
            outcome.degraded.as_ref().map(|d| d.reason.clone()),
            Some(DegradationReason::EmbeddingsUnavailable(
                "OPENROUTER_API_KEY is not set".to_string()
            ))
        );
        assert_eq!(
            outcome.hits.len(),
            1,
            "keyword results survive the fallback"
        );
        Ok(())
    }

    #[test]
    fn test_failing_semantic_call_keeps_keyword_hits() {
        // The client exists but the API call fails — network down, quota hit, bad
        // key. This is the likeliest failure, and it must not fail the search.
        let keyword_hits = vec![SearchHit {
            id: FactId::new(),
            scope: Scope::Global,
            project_name: String::new(),
            fact_type: FactType::Decision,
            title: "Keyword only".to_string(),
            description: None,
            snippet: String::new(),
            tags: Vec::new(),
            status: FactStatus::Stable,
            supersedes: None,
            file_path: None,
            score: 1.0,
        }];

        let outcome = combine(
            keyword_hits,
            Err(anyhow::anyhow!("503 Service Unavailable")),
            10,
        );

        let degradation = outcome.degraded.as_ref().expect("must be reported");
        assert_eq!(
            degradation.reason,
            DegradationReason::SemanticSearchFailed("503 Service Unavailable".to_string())
        );
        assert_eq!(outcome.hits.len(), 1);
        assert_eq!(outcome.hits[0].title, "Keyword only");
    }

    #[test]
    fn test_successful_semantic_half_is_not_reported_as_degraded() {
        let outcome = combine(Vec::new(), Ok(Vec::new()), 10);
        assert!(!outcome.is_degraded());
    }
}
