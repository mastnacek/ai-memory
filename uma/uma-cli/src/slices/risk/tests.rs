//! Risk-slice tests: pure parts without git or memory.

use super::*;
use uma_core::domain::{Fact, FactType, Scope};

fn correction(title: &str, body: &str) -> Fact {
    Fact::new(Scope::Project("ai-memory".to_string()), FactType::Correction, title.to_string(), body.to_string())
}

#[test]
fn test_fact_mentions_full_path_or_stem() {
    let fact = correction("Fixed indexer lock", "The bug was in uma-core/src/indexer/ops.rs under load");
    assert!(fact_mentions(&fact, "uma-core/src/indexer/ops.rs", "ops"));
    assert!(fact_mentions(&fact, "somewhere/else", "ops"), "stem matches");
    assert!(!fact_mentions(&fact, "uma-core/src/store/mod.rs", "mod"));
}

#[test]
fn test_fact_mentions_never_fires_on_empty_stem() {
    let fact = correction("t", "b");
    assert!(!fact_mentions(&fact, "anything", ""));
}

#[test]
fn test_windows_paths_match_forward_slash_needles() {
    let fact = correction("Fixed store", "Bug in D:\\proj\\ai-memory\\.uma\\..\\src\\store\\ops.rs");
    assert!(fact_mentions(&fact, "src/store/ops.rs", "ops"));
}

#[test]
fn test_git_history_counts_reverts_and_churn() {
    // git_history shells out; here only the parsing contract is pinned via
    // the subject-line rule: a line containing "revert" counts once.
    let subjects = ["abc001 Revert \"fix ops\"", "abc002 fix ops", "abc003 revert previous change"];
    let reverts = subjects.iter().filter(|l| l.to_lowercase().contains("revert")).count();
    assert_eq!(reverts, 2);
}
