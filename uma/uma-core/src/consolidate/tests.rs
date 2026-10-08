//! Consolidation tests: pure, offline, no store required.

use super::*;
use crate::domain::{FactType, Scope};

fn fact(title: &str, body: &str) -> Fact {
    Fact::new(
        Scope::Project("demo".to_string()),
        FactType::Decision,
        title.to_string(),
        body.to_string(),
    )
}

#[test]
fn test_near_duplicate_is_proposed_for_merge() {
    // The first body deliberately says "not npm": a comparative aside must
    // not be mistaken for a polarity flip and turn a merge into a conflict.
    let facts = vec![
        fact(
            "Use pnpm for dependency installs",
            "Install dependencies with pnpm, not npm.",
        ),
        fact(
            "Use pnpm for dependency installation",
            "Dependencies are installed with pnpm in this repo.",
        ),
    ];

    let report = analyze(&facts, &AnalyzeOptions::default());

    assert_eq!(report.scanned, 2);
    assert_eq!(
        report.duplicate_groups.len(),
        1,
        "expected one merge proposal"
    );
    let group = &report.duplicate_groups[0];
    assert_eq!(group.ids.len(), 2);
    assert!(group.score >= 0.55, "score was {}", group.score);
    assert!(report.contradictions.is_empty());
}

#[test]
fn test_opposing_facts_are_flagged_as_contradiction() {
    let facts = vec![
        fact(
            "Use pnpm for dependency installs",
            "Install dependencies with pnpm.",
        ),
        fact(
            "Do not use pnpm for dependency installs",
            "Never install dependencies with pnpm.",
        ),
    ];

    let report = analyze(&facts, &AnalyzeOptions::default());

    assert_eq!(report.contradictions.len(), 1, "expected one contradiction");
    assert_eq!(
        report.contradictions[0].reason,
        "similar wording with opposing polarity"
    );
    // A conflict must never also be offered as a merge.
    assert!(report.duplicate_groups.is_empty());
}

#[test]
fn test_unrelated_facts_produce_no_proposals() {
    let facts = vec![
        fact(
            "Use pnpm for dependency installs",
            "Install dependencies with pnpm.",
        ),
        fact(
            "Postgres handles connection pooling",
            "The database layer pools connections.",
        ),
    ];

    let report = analyze(&facts, &AnalyzeOptions::default());
    assert_eq!(report.scanned, 2);
    assert!(report.is_empty(), "unrelated facts must not be grouped");
}

#[test]
fn test_deprecated_facts_are_excluded_by_default() {
    let mut old = fact("Use pnpm for dependency installs", "Install with pnpm.");
    old.status = crate::domain::FactStatus::Deprecated;
    old.validity.until = Some(chrono::Utc::now());
    let current = fact(
        "Use pnpm for dependency installation",
        "Dependencies are installed with pnpm.",
    );

    let default_report = analyze(&[old.clone(), current.clone()], &AnalyzeOptions::default());
    assert_eq!(default_report.scanned, 1);
    assert!(default_report.is_empty());

    let inclusive = AnalyzeOptions {
        include_deprecated: true,
        ..Default::default()
    };
    let full_report = analyze(&[old, current], &inclusive);
    assert_eq!(full_report.scanned, 2);
    assert_eq!(full_report.duplicate_groups.len(), 1);
}
