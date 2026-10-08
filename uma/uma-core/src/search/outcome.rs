//! Search outcome types: what actually ran, versus what was asked for.
//!
//! These exist so a degradation is part of the *return value* rather than a log
//! line. A logging-only signal can be dropped anywhere along the way to the
//! user, and the caller is then left believing it received hybrid ranking when
//! it received BM25 — confidently wrong, and invisible.

use super::{SearchHit, SearchMode};

/// The result of a search, plus whether it ran the way the caller asked.
#[derive(Debug, Clone)]
pub struct SearchOutcome {
    pub hits: Vec<SearchHit>,
    /// `None` when the requested mode was honoured in full.
    pub degraded: Option<Degradation>,
}

impl SearchOutcome {
    /// A search that did exactly what was requested.
    pub fn complete(hits: Vec<SearchHit>) -> Self {
        Self {
            hits,
            degraded: None,
        }
    }

    pub fn is_degraded(&self) -> bool {
        self.degraded.is_some()
    }

    /// A one-line explanation for the caller, or `None` if nothing degraded.
    pub fn note(&self) -> Option<String> {
        self.degraded.as_ref().map(Degradation::message)
    }
}

/// Records that a requested mode could not be fully honoured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Degradation {
    pub requested: SearchMode,
    pub performed: SearchMode,
    pub reason: DegradationReason,
}

impl Degradation {
    pub fn message(&self) -> String {
        match &self.reason {
            DegradationReason::EmbeddingsUnavailable(err) => format!(
                "{:?} search degraded to {:?}: embedding client unavailable ({err}). \
                 These results are keyword-only; run `uma search \"\" --vectorize` and check \
                 the embedding API key.",
                self.requested, self.performed
            ),
            DegradationReason::SemanticSearchFailed(err) => format!(
                "{:?} search degraded to {:?}: semantic ranking failed ({err}). \
                 These results are keyword-only; run `uma search \"\" --vectorize` and check \
                 the embedding API key.",
                self.requested, self.performed
            ),
            DegradationReason::NoStoredEmbeddings => format!(
                "{:?} search degraded to {:?}: no embeddings are stored. \
                 These results are keyword-only; run `uma search \"\" --vectorize`.",
                self.requested, self.performed
            ),
            // The relaxation degrades PRECISION, not the mode: say so instead
            // of the vectorize advice, which does not apply here.
            DegradationReason::QueryRelaxedToOr => format!(
                "{:?} search relaxed to any-term matching: no fact contained every \
                 query term. Results may be loosely related; narrow the query for \
                 precision.",
                self.requested
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DegradationReason {
    /// No embedding client could be constructed (missing key, bad config).
    EmbeddingsUnavailable(String),
    /// The client was built but the call failed: network, quota, or a bad key.
    /// Without handling this, the whole search would fail even though BM25 works.
    SemanticSearchFailed(String),
    /// Nothing is vectorized, so the semantic half cannot contribute.
    NoStoredEmbeddings,
    /// The all-terms query matched nothing, so the search was retried with
    /// any-term (OR) matching and those hits were returned instead.
    QueryRelaxedToOr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_outcome_carries_no_note() {
        let outcome = SearchOutcome::complete(Vec::new());
        assert!(!outcome.is_degraded());
        assert!(outcome.note().is_none());
    }

    #[test]
    fn test_message_names_the_cause_and_the_remedy() {
        let message = Degradation {
            requested: SearchMode::Hybrid,
            performed: SearchMode::Keyword,
            reason: DegradationReason::NoStoredEmbeddings,
        }
        .message();
        assert!(message.contains("no embeddings are stored"), "{message}");
        assert!(message.contains("--vectorize"), "{message}");
    }

    #[test]
    fn test_message_distinguishes_client_failure_from_call_failure() {
        let unavailable = Degradation {
            requested: SearchMode::Hybrid,
            performed: SearchMode::Keyword,
            reason: DegradationReason::EmbeddingsUnavailable("no key".to_string()),
        }
        .message();
        let failed = Degradation {
            requested: SearchMode::Hybrid,
            performed: SearchMode::Keyword,
            reason: DegradationReason::SemanticSearchFailed("503".to_string()),
        }
        .message();

        assert!(unavailable.contains("client unavailable"), "{unavailable}");
        assert!(failed.contains("ranking failed"), "{failed}");
        // Both must still tell the caller the results are keyword-only.
        assert!(unavailable.contains("keyword-only"));
        assert!(failed.contains("keyword-only"));
    }
}
