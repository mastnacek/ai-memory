//! Consolidation analysis: groups near-identical facts and flags opposing ones.
//!
//! This module only *proposes*. Nothing here mutates the store — accepting a
//! proposal is the caller's job, because applying a merge or resolving a
//! contradiction is a memory write and therefore needs consent. A consuming
//! agent reviews the report and then calls `uma_supersede` (to retire the
//! losing fact) or `uma_write` (to state the merged result).
//!
//! Precision is intentionally traded for recall: a false proposal costs one
//! human glance, whereas a missed duplicate silently rots into two competing
//! truths. The threshold is therefore a dial, not a verdict.

use std::collections::HashMap;

use serde::Serialize;

use crate::domain::{Fact, FactId};
use crate::similarity::{has_negation, jaccard, tokenize};

/// A set of facts that appear to state the same thing.
#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    pub ids: Vec<FactId>,
    pub titles: Vec<String>,
    /// Mean pairwise similarity within the group (0.0-1.0).
    pub score: f64,
    pub reason: String,
}

/// Two facts that look alike but disagree.
#[derive(Debug, Clone, Serialize)]
pub struct ContradictionPair {
    pub left: FactId,
    pub right: FactId,
    pub left_title: String,
    pub right_title: String,
    pub score: f64,
    pub reason: String,
}

/// The full, purely descriptive result of an analysis pass.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ConsolidationReport {
    /// How many active facts were compared.
    pub scanned: usize,
    pub duplicate_groups: Vec<DuplicateGroup>,
    pub contradictions: Vec<ContradictionPair>,
}

impl ConsolidationReport {
    /// True when the pass found nothing worth reviewing.
    pub fn is_empty(&self) -> bool {
        self.duplicate_groups.is_empty() && self.contradictions.is_empty()
    }
}

/// Tunable inputs for an analysis pass.
#[derive(Debug, Clone)]
pub struct AnalyzeOptions {
    /// Minimum blended similarity before two facts are considered related.
    pub threshold: f64,
    /// Weight of title similarity; the remainder applies to body similarity.
    pub title_weight: f64,
    /// Ignore stale/inactive facts (the default for every other read path).
    pub include_deprecated: bool,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            threshold: 0.55,
            title_weight: 0.6,
            include_deprecated: false,
        }
    }
}

/// Per-fact token cache, so each body is tokenized exactly once.
struct Fingerprint {
    title: Vec<String>,
    body: Vec<String>,
    negated: bool,
}

fn find(parent: &mut [usize], mut idx: usize) -> usize {
    while parent[idx] != idx {
        parent[idx] = parent[parent[idx]];
        idx = parent[idx];
    }
    idx
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra != rb {
        parent[rb] = ra;
    }
}

/// Compares every active fact against every other and returns proposals.
///
/// O(n²) by design: the fact set is expected to be in the hundreds, and a
/// quadratic scan over plain token sets is far cheaper than maintaining an
/// approximate-nearest-neighbour index for a personal memory store.
pub fn analyze(facts: &[Fact], opts: &AnalyzeOptions) -> ConsolidationReport {
    let now = chrono::Utc::now();
    let active: Vec<&Fact> = facts
        .iter()
        .filter(|f| opts.include_deprecated || f.is_active_at(now))
        .collect();

    let fingerprints: Vec<Fingerprint> = active
        .iter()
        .map(|f| Fingerprint {
            title: tokenize(&f.title),
            body: tokenize(&f.body),
            // Polarity is read from the title only. Titles state the claim;
            // bodies routinely contain comparative asides ("use pnpm, not npm")
            // that would otherwise register as false contradictions.
            negated: has_negation(&f.title),
        })
        .collect();

    let count = active.len();
    let mut report = ConsolidationReport {
        scanned: count,
        ..Default::default()
    };

    let mut parent: Vec<usize> = (0..count).collect();
    let mut agreements: Vec<((usize, usize), f64)> = Vec::new();

    for i in 0..count {
        for j in (i + 1)..count {
            let title_score = jaccard(&fingerprints[i].title, &fingerprints[j].title);
            let body_score = jaccard(&fingerprints[i].body, &fingerprints[j].body);
            let score = opts.title_weight * title_score + (1.0 - opts.title_weight) * body_score;

            if score < opts.threshold {
                continue;
            }

            // Opposing polarity means the pair is a conflict to resolve, not a
            // duplicate to merge — merging them would destroy the disagreement.
            if fingerprints[i].negated != fingerprints[j].negated {
                report.contradictions.push(ContradictionPair {
                    left: active[i].id.clone(),
                    right: active[j].id.clone(),
                    left_title: active[i].title.clone(),
                    right_title: active[j].title.clone(),
                    score,
                    reason: "similar wording with opposing polarity".to_string(),
                });
                continue;
            }

            union(&mut parent, i, j);
            agreements.push(((i, j), score));
        }
    }

    // Collect connected components; a component of 2+ facts is a merge proposal.
    let mut components: HashMap<usize, Vec<usize>> = HashMap::new();
    for idx in 0..count {
        let root = find(&mut parent, idx);
        components.entry(root).or_default().push(idx);
    }

    for members in components.values() {
        if members.len() < 2 {
            continue;
        }
        let mut sum = 0.0;
        let mut pairs = 0usize;
        for ((i, j), score) in &agreements {
            if members.contains(i) && members.contains(j) {
                sum += score;
                pairs += 1;
            }
        }
        report.duplicate_groups.push(DuplicateGroup {
            ids: members.iter().map(|&i| active[i].id.clone()).collect(),
            titles: members.iter().map(|&i| active[i].title.clone()).collect(),
            score: if pairs > 0 { sum / pairs as f64 } else { 0.0 },
            reason: "overlapping title/body wording".to_string(),
        });
    }

    report
        .duplicate_groups
        .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    report
        .contradictions
        .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    report
}

#[cfg(test)]
mod tests {
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
        assert_eq!(report.duplicate_groups.len(), 1, "expected one merge proposal");
        let group = &report.duplicate_groups[0];
        assert_eq!(group.ids.len(), 2);
        assert!(group.score >= 0.55, "score was {}", group.score);
        assert!(report.contradictions.is_empty());
    }

    #[test]
    fn test_opposing_facts_are_flagged_as_contradiction() {
        let facts = vec![
            fact("Use pnpm for dependency installs", "Install dependencies with pnpm."),
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
            fact("Use pnpm for dependency installs", "Install dependencies with pnpm."),
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
}
