//! Search entry point and index-cache maintenance.

use crate::domain::{FactType, Scope};
use crate::embeddings::EmbeddingClient;
use crate::search::{
    search_hybrid_resilient, search_keyword, search_semantic, SearchMode, SearchOptions,
    SearchOutcome,
};
use anyhow::Result;
use chrono::{DateTime, Utc};

use super::Store;

impl Store {
    /// Performs search using BM25 keyword matching, Semantic vector search, or Hybrid RRF fusion.
    pub fn search_all(
        query: &str,
        scope: Option<&Scope>,
        fact_type: Option<&FactType>,
        mode: SearchMode,
        include_deprecated: bool,
        as_of: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<SearchOutcome> {
        let indexer = Self::central_indexer()?;
        let current_project = match scope {
            Some(_) => None,
            None => Self::current_project_name(),
        };

        let opts = SearchOptions {
            query,
            scope,
            current_project: current_project.as_deref(),
            fact_type,
            include_deprecated,
            as_of,
            limit,
        };

        match mode {
            SearchMode::Keyword => {
                let hits = search_keyword(indexer.connection(), &opts)?;
                if hits.is_empty() && indexer.indexed_count()? == 0 {
                    // An empty index is a cache that was never built (or was
                    // wiped), so build it and retry once. An empty RESULT on a
                    // populated index is a genuine miss — rebuilding there
                    // wiped and re-parsed the whole store on every typo.
                    let indexed = Self::reindex_all()?;
                    if indexed > 0 {
                        return Ok(SearchOutcome::complete(search_keyword(
                            indexer.connection(),
                            &opts,
                        )?));
                    }
                }
                Ok(SearchOutcome::complete(hits))
            }
            SearchMode::Semantic => {
                // Asked for semantic only: fail rather than silently substitute
                // keyword ranking, so the caller can choose what to do.
                let client = EmbeddingClient::new(None)?;
                Ok(SearchOutcome::complete(search_semantic(
                    indexer.connection(),
                    &opts,
                    &client,
                )?))
            }
            SearchMode::Hybrid => {
                // Falls back to keyword-only when embeddings are unavailable, and
                // reports it in the outcome — see `search_hybrid_resilient`.
                search_hybrid_resilient(indexer.connection(), &opts)
            }
        }
    }

    /// Rebuilds the centralized SQLite index by scanning global and project stores.
    pub fn reindex_all() -> Result<usize> {
        let indexer = Self::central_indexer()?;
        let mut dirs_to_scan = Vec::new();

        if let Ok(global_store) = Self::global() {
            dirs_to_scan.push(global_store.root);
        }

        if let Ok(git_root) = Self::find_git_root() {
            let project_uma = git_root.join(".uma");
            if project_uma.exists() {
                dirs_to_scan.push(project_uma);
            }
        }

        indexer.reindex_from_dirs(&dirs_to_scan)
    }

    /// Generates vector embeddings for all facts that are currently missing them.
    pub fn vectorize_all() -> Result<usize> {
        let indexer = Self::central_indexer()?;
        let client = EmbeddingClient::new(None)?;
        indexer.vectorize_missing(&client)
    }
}
