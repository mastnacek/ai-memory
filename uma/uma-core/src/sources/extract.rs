//! Candidate extraction from agent sessions.
//!
//! A session is conversation; a fact is one atomic, durable claim. This module
//! proposes which turns are worth keeping, and classifies them by *pattern of
//! language* rather than by understanding them — classification and translation
//! into proper fact titles/bodies is the reviewing agent's job, done with the
//! operator watching through the approval modal. What this extractor guarantees
//! is that the pool it produces is worth reviewing at all.
//!
//! Read-only by construction: candidates describe the session, nothing here
//! touches the store.

use serde::Serialize;

use super::types::SessionDetail;

/// The fact kinds worth importing from a conversation.
///
/// Preference is included because session instructions ("always do X") are how
/// operators actually state preferences — even though the store's quality rules
/// say a preference belongs in global scope, so the reviewing agent may re-scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CandidateKind {
    Decision,
    Correction,
    Preference,
    Note,
}

impl CandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CandidateKind::Decision => "decision",
            CandidateKind::Correction => "correction",
            CandidateKind::Preference => "preference",
            CandidateKind::Note => "note",
        }
    }
}

/// One proposed memory candidate, with its session provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    pub kind: CandidateKind,
    /// Suggested title, trimmed from the message itself. The reviewing agent is
    /// expected to improve it (and translate it) before proposing.
    pub suggested_title: String,
    /// The substantive user turn the candidate came from, verbatim.
    pub quote: String,
    /// Session provenance: the fact's `since` should be the session's start.
    pub session_id: String,
    pub project: String,
}

/// Turns that are conversation rather than direction, matched on lowercase text.
const SMALL_TALK: &[&str] = &[
    "how are you",
    "jak se mas",
    "jak se ti",
    "thank you",
    "dekuji",
    "diky",
    "hello",
    "ahoj",
    "zdravim",
    "good morning",
    "who are you",
    "kdo jsi",
    "test",
];

/// Fragments that mark a turn as an instruction about the *session* itself,
/// which is ephemeral and must not become durable memory.
const META_COMMAND: &[&str] = &[
    "continue",
    "pokracuj",
    "go on",
    "stop",
    "zastav",
    "retry",
    "zkus znovu",
    "be quiet",
    "nic nerikej",
    "run the tests again",
];

/// Language that marks a turn as a preference rather than a one-off decision.
const PREFERENCE_MARKERS: &[&str] = &[
    "always ",
    "vzdy ",
    "never ",
    "nikdy ",
    "prefer ",
    "preferuji",
    "preferuju",
    "chci vzdy",
    "do not use ",
    "nepouzivej",
    "from now on",
    "od ted",
    "pamatuj si ze",
];

/// Language that marks a correction: a mistaken path, assumption, or approach.
const CORRECTION_MARKERS: &[&str] = &[
    "that's wrong",
    "to je spatne",
    "spatne,",
    "not there",
    "tam ne",
    "ne tam",
    "mistake",
    "chyba",
    "instead of",
    "misto toho",
    "neni to",
    "nefunguje",
    "nefunguje to",
    "proc to",
    "why does not",
    "why doesn't",
    "proc nefunguje",
];

/// Language that marks an architectural or procedural decision.
const DECISION_MARKERS: &[&str] = &[
    "should ",
    "musime",
    "musis",
    "use ",
    "pouzij",
    "pouzivej",
    "implement",
    "implementuj",
    "refactor",
    "pridat",
    "vytvor",
    "udelej",
    "architekt",
    "slice",
    "we will",
    "bude to",
    "nechci",
    "chci",
];

/// True when the turn is small talk or a session-control command.
fn is_noise(lower: &str) -> bool {
    SMALL_TALK.iter().any(|m| lower.starts_with(m))
        || META_COMMAND.iter().any(|m| lower == *m)
        || lower.len() < 12
}

/// Classifies one substantive user turn.
fn classify(lower: &str) -> CandidateKind {
    if PREFERENCE_MARKERS.iter().any(|m| lower.contains(m)) {
        CandidateKind::Preference
    } else if CORRECTION_MARKERS.iter().any(|m| lower.contains(m)) {
        CandidateKind::Correction
    } else if DECISION_MARKERS.iter().any(|m| lower.contains(m)) {
        CandidateKind::Decision
    } else {
        CandidateKind::Note
    }
}

/// A suggested title: the first sentence, trimmed to something readable.
fn suggest_title(text: &str) -> String {
    let first = text
        .split(|c: char| c == '.' || c == '?' || c == '!' || c == '\n')
        .find(|s| !s.trim().is_empty())
        .unwrap_or(text);
    let mut title = first.trim().to_string();
    if title.len() > 80 {
        title.truncate(80);
        title.push_str("...");
    }
    title
}

/// Jaccard threshold above which two titles are the same instruction said
/// twice. Matches the consolidator's default proposal threshold so both
/// proposers agree on what "duplicate" means.
const CROSS_SESSION_DUPLICATE: f64 = 0.55;

/// The near-identical prefix comparison for the per-session pass: exact
/// repeats differ by a trailing word or one truncated character, not substance.
fn near_identical(a: &str, b: &str) -> bool {
    let overlap = a.len().min(b.len());
    let (x, y) = (&a.as_bytes()[..overlap], &b.as_bytes()[..overlap]);
    x.iter().zip(y).filter(|(p, q)| p != q).count() <= 1
}

/// Extracts memory candidates from one session's substantive user turns.
///
/// Turns are de-duplicated by their first 60 characters (an operator often
/// repeats an instruction), and `max_per_session` bounds how many candidates
/// one session may contribute — a 20-turn session is a conversation, not a
/// bag of facts, and the strongest signal is usually at its start.
pub fn candidates_from(detail: &SessionDetail, max_per_session: usize) -> Vec<Candidate> {
    let mut seen: Vec<String> = Vec::new();
    let mut candidates = Vec::new();

    for message in &detail.user_messages {
        if candidates.len() >= max_per_session {
            break;
        }
        let lower = format!("{} ", message.to_lowercase());
        if is_noise(&lower) {
            continue;
        }
        let key: String = lower.chars().take(60).collect();
        // A repeat counts when the two keys are near-identical, compared with
        // tolerance for a single differing tail character - truncation can
        // otherwise cut one string before a comma and the other after it,
        // hiding an exact repeat ("…module " vs "…module,").
        if seen.iter().any(|s| near_identical(s, &key)) {
            continue;
        }
        seen.push(key);

        candidates.push(Candidate {
            kind: classify(&lower),
            suggested_title: suggest_title(message),
            quote: message.clone(),
            session_id: detail.record.session_id.clone(),
            project: detail.record.project.clone(),
        });
    }
    candidates
}

/// Extracts candidates from many sessions, flattening their provenance.
///
/// The same instruction often repeats across a project's sessions with small
/// wording changes ("fine-tune the theme" / "fine-tune and perfect the theme"),
/// so the pool is de-duplicated across sessions by near-identical title. The
/// first occurrence wins and keeps its own provenance.
pub fn candidates_from_many(
    details: &[SessionDetail],
    max_per_session: usize,
) -> Vec<Candidate> {
    let mut seen_titles: Vec<Vec<String>> = Vec::new();
    let mut candidates = Vec::new();
    for detail in details {
        for candidate in candidates_from(detail, max_per_session) {
            // Cross-session repeats rephrase ("fine-tune the theme" vs
            // "fine-tune and perfect the theme"), so compare token bags with
            // the kernel's Jaccard — the same primitive the consolidator uses
            // for proposing merges — rather than byte prefixes.
            let tokens = crate::similarity::tokenize(&candidate.suggested_title);
            if seen_titles.iter().any(|s| {
                crate::similarity::jaccard(s, &tokens) >= CROSS_SESSION_DUPLICATE
            }) {
                continue;
            }
            seen_titles.push(tokens);
            candidates.push(candidate);
        }
    }
    candidates
}

#[cfg(test)]
mod tests;
