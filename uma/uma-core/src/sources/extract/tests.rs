//! Candidate-extraction tests: pure, fixture-driven.

use super::*;
use crate::sources::types::{SessionRecord, SessionSource};
use chrono::Utc;
use std::path::PathBuf;

fn detail(user_messages: &[&str]) -> SessionDetail {
    SessionDetail {
        record: SessionRecord {
            source: SessionSource::PiAgent,

            project: "D:\\proj\\demo".to_string(),

            session_id: "01a03770".to_string(),

            started_at: Utc::now(),

            title: None,

            file: PathBuf::from("x.jsonl"),

            size_bytes: 1,

            user_messages: user_messages.len(),

            assistant_messages: 1,

            substantive_user_turns: user_messages.len(),
        },

        user_messages: user_messages.iter().map(|s| s.to_string()).collect(),

        assistant_messages: 1,

        tool_results: 0,
    }
}

#[test]

fn test_greetings_and_meta_commands_are_skipped() {
    let detail = detail(&[
        "How are you today?",
        "jak se ti libi ubuntu?",
        "Continue",
        "A decision worth keeping: we will store the FTS index in the user profile.",
    ]);

    let candidates = candidates_from(&detail, 10);

    assert_eq!(candidates.len(), 1, "{candidates:#?}");

    assert_eq!(candidates[0].kind, CandidateKind::Decision);
}

#[test]

fn test_preferences_and_corrections_are_classified() {
    let detail = detail(&[
        "Always use pnpm, never npm, in this repository.",
        "nefunguje to, misto toho pouzij tempfile",
    ]);

    let candidates = candidates_from(&detail, 10);

    assert_eq!(candidates[0].kind, CandidateKind::Preference);

    assert_eq!(candidates[1].kind, CandidateKind::Correction);
}

#[test]

fn test_repeated_instructions_are_deduplicated() {
    let detail = detail(&[
        "Always write tests before refactoring the indexer module",
        "Always write tests before refactoring the indexer module, please",
    ]);

    assert_eq!(candidates_from(&detail, 10).len(), 1);
}

#[test]

fn test_max_per_session_bounds_the_pool() {
    // Distinct turns: a repeated instruction is deduplicated elsewhere, so
    // bounding the pool only means something when every turn differs.
    // Turn 1..=6 differ within the 60-char dedup window, so each counts;
    // the bound is what caps the pool, not repetition.
    let turns: Vec<String> = (1..=6)
        .map(|i| {
            let subjects = [
                "dependency installs",
                "the theme switching",
                "the slice isolation",
            ];
            let verbs = ["Use pnpm for", "Implement", "Refactor"];
            format!(
                "{} {} -- task {}: {} needs its own slice and tests",
                verbs[i % 3],
                subjects[i % 3],
                i,
                ["indexing", "themes", "modules", "search", "sync", "export"][i % 6]
            )
        })
        .collect();
    let turns: Vec<&str> = turns.iter().map(String::as_str).collect();
    let detail = detail(&turns);

    assert_eq!(
        candidates_from(&detail, 3).len(),
        3,
        "the bound caps the pool"
    );
    assert_eq!(
        candidates_from(&detail, 100).len(),
        6,
        "all distinct turns count"
    );
}

#[test]

fn test_candidates_carry_session_provenance() {
    let detail = detail(&["We will store the index in the user profile."]);

    let candidates = candidates_from(&detail, 5);

    assert_eq!(candidates[0].session_id, "01a03770");

    assert_eq!(candidates[0].project, "D:\\proj\\demo");
}
#[test]
fn test_cross_session_repeats_are_deduplicated() {
    let a = detail(&["Fine-tune and perfect the linkarzu theme."]);
    let b = detail(&["Fine-tune the Linkarzu theme to perfection."]);

    let pool = candidates_from_many(&[a, b], 5);
    assert_eq!(
        pool.len(),
        1,
        "the same instruction across sessions is one candidate"
    );
}
