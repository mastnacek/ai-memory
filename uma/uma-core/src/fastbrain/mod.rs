//! Fastbrain: the System-1 triage layer.
//!
//! UMA's kernel already contains deterministic, offline triage — the
//! consolidator's lexical similarity, the import extractor's language
//! markers. This module formalizes the *other* half of the two-speed brain:
//! when a judgment is semantic (is this a contradiction? does this message
//! need memory?), a fast typed-question model answers it, and code branches
//! on the typed answer instead of parsing prose.
//!
//! **Two transports, one contract:**
//! - [`offline`] — the deterministic heuristics UMA ships with. Always
//!   available, zero network, what `cargo test` runs. Every semantic question
//!   degrades to this.
//! - [`jev`] — TypeSafe's Jev (System One model) via OpenRouter chat
//!   completions with a JSON-schema response, emulating the typed-primitive
//!   contract (`choice`/`noul`/`score`) until a native TypeSafe API key is
//!   configured. Same contract, swappable transport.
//!
//! **Invariants this module must not break:** a judgment is advisory — code
//! proposes, the operator consents through the modal; a failed judge call
//! degrades to the offline answer visibly, never aborts a slice; and no
//! secret ever reaches a judge (the secrets gate runs first on anything a
//! judge would see).

pub mod jev;
pub mod offline;

use serde::{Deserialize, Serialize};

/// How two near-identical facts relate — the consolidation judge's output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relationship {
    /// Same rule said twice — one should supersede the other.
    Duplicate,
    /// Opposite rules — the later one wins through supersession.
    Contradiction,
    /// Different rules that happen to share vocabulary.
    Unrelated,
}

impl Relationship {
    pub fn as_str(self) -> &'static str {
        match self {
            Relationship::Duplicate => "duplicate",
            Relationship::Contradiction => "contradiction",
            Relationship::Unrelated => "unrelated",
        }
    }
}

/// A typed judgment: the answer plus 0.0–1.0 confidence in it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Judgment<T> {
    pub answer: T,
    pub confidence: f64,
    /// Which transport produced this — callers surface it, because a judged
    /// answer and a guessed answer must never look the same in output.
    pub judged_by: Backend,
    /// Set when a Jev answer degraded to offline, or when the transport
    /// carries provenance worth surfacing.
    pub notes: Option<String>,
}

/// Which fastbrain transport answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Offline,
    Jev,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::Offline => "offline",
            Backend::Jev => "jev",
        }
    }
}

/// Which transport a slice asked for, parsed from CLI args.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Judge {
    /// Deterministic heuristics only (the default: no key, no network).
    #[default]
    Offline,
    /// Jev via OpenRouter; degrades to offline with a visible note.
    Jev,
}

/// Classifies the relationship between two fact texts.
///
/// The offline answer is deterministic (negation difference, token overlap);
/// the Jev answer is semantic. Callers decide what to *do* with a
/// contradiction verdict — for UMA that is always a supersession proposal
/// through the modal, never an automatic rewrite.
pub fn judge_relationship(
    text_a: &str,
    text_b: &str,
    judge: Judge,
) -> Result<Judgment<Relationship>, String> {
    match judge {
        Judge::Offline => offline::relationship(text_a, text_b),
        Judge::Jev => match jev::relationship(text_a, text_b) {
            Ok(judgment) => Ok(judgment),
            Err(failure) => {
                let mut degraded = offline::relationship(text_a, text_b)?;
                degraded.judged_by = Backend::Offline;
                degraded.confidence *= 0.5;
                // The failure is the caller's to surface; the degraded answer
                // is still better than nothing, at half confidence.
                degraded.notes = Some(format!("Jev unavailable ({failure}); offline verdict"));
                Ok(degraded)
            }
        },
    }
}

/// Given a user message, decides whether memory recall is worth a search and
/// which fact types to search — the S3 recall gate's contract.
///
/// Offline is marker-based and cheap; Jev is semantic. Either way the answer
/// is advisory: the caller (Pi plugin) still respects the operator's S3
/// on/off decision.
pub fn judge_recall_need(message: &str, judge: Judge) -> Result<Judgment<RecallNeed>, String> {
    match judge {
        Judge::Offline => offline::recall_need(message),
        Judge::Jev => match jev::recall_need(message) {
            Ok(judgment) => Ok(judgment),
            Err(failure) => {
                let mut degraded = offline::recall_need(message)?;
                degraded.judged_by = Backend::Offline;
                degraded.confidence *= 0.5;
                degraded.notes = Some(format!("Jev unavailable ({failure}); offline verdict"));
                Ok(degraded)
            }
        },
    }
}

/// The recall gate's answer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecallNeed {
    /// Search memory before answering.
    pub search: bool,
    /// Which fact types to search, most relevant first.
    pub fact_types: Vec<String>,
    pub notes: Option<String>,
}
