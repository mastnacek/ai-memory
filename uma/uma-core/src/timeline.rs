//! Supersession chains: reconstructing how a fact evolved.
//!
//! Each revision points at its predecessor through `supersedes`, so facts form
//! chains rather than a flat list. These helpers walk those links to rebuild the
//! history. **Read-only** — nothing here mutates the store.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use crate::domain::{Fact, FactId, FactStatus};

/// One revision within a chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainStep {
    pub id: FactId,
    pub title: String,
    pub status: FactStatus,
    /// When the claim's substance started to hold — inherited across
    /// supersession, so every step of a chain can share one origin.
    pub since: DateTime<Utc>,
    /// When this revision was written (its `generated.at`); the per-step
    /// timestamp of revision history. Falls back to `since` for facts whose
    /// frontmatter carries no `generated` event.
    pub revised_at: DateTime<Utc>,
    pub until: Option<DateTime<Utc>>,
}

/// A supersession chain, oldest revision first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    pub steps: Vec<ChainStep>,
}

impl Chain {
    /// The newest revision in the chain.
    pub fn tip(&self) -> Option<&ChainStep> {
        self.steps.last()
    }

    /// The revision that is still active, if any.
    pub fn active(&self) -> Option<&ChainStep> {
        self.steps
            .iter()
            .find(|step| step.status == FactStatus::Stable)
    }

    /// True when at least one revision was replaced (so there is history here).
    pub fn has_history(&self) -> bool {
        self.steps.len() > 1
    }
}

fn step_of(fact: &Fact) -> ChainStep {
    ChainStep {
        id: fact.id,
        title: fact.title.clone(),
        status: fact.status,
        since: fact.validity.since,
        revised_at: fact.generated.as_ref().map(|g| g.at).unwrap_or(fact.validity.since),
        until: fact.validity.until,
    }
}

/// Maps each predecessor to its newest successor.
///
/// A fact can in principle be superseded more than once; the newest revision
/// wins so the chain stays linear and readable.
fn successor_map<'a>(
    facts: &'a [Fact],
    by_id: &HashMap<FactId, &'a Fact>,
) -> HashMap<FactId, FactId> {
    let mut successors: HashMap<FactId, FactId> = HashMap::new();

    for fact in facts {
        let Some(predecessor) = fact.supersedes else {
            continue;
        };
        if !by_id.contains_key(&predecessor) {
            continue;
        }
        // "Newer" means written later, not claiming a later origin: with
        // `since` inherited across revisions, ordering by it would be a tie.
        let incumbent_is_newer = successors
            .get(&predecessor)
            .and_then(|existing| by_id.get(existing))
            .map(|existing| step_of(existing).revised_at >= step_of(fact).revised_at)
            .unwrap_or(false);

        if !incumbent_is_newer {
            successors.insert(predecessor, fact.id);
        }
    }

    successors
}

/// Groups `facts` into supersession chains, oldest revision first.
///
/// Chains are ordered newest-first, so the most recently revised history appears
/// at the top. Chains of a single step — facts that were never superseded — are
/// omitted unless `include_singletons` is set, which is what keeps unfiltered
/// timeline output quiet.
pub fn build_chains(facts: &[Fact], include_singletons: bool) -> Vec<Chain> {
    let by_id: HashMap<FactId, &Fact> = facts.iter().map(|fact| (fact.id, fact)).collect();
    let successors = successor_map(facts, &by_id);

    // A chain starts where no predecessor exists, or where the recorded
    // predecessor is not in this fact set (an orphaned revision).
    let roots: Vec<&Fact> = facts
        .iter()
        .filter(|fact| match fact.supersedes {
            None => true,
            Some(predecessor) => !by_id.contains_key(&predecessor),
        })
        .collect();

    let mut chains: Vec<Chain> = Vec::new();
    let mut visited: HashSet<FactId> = HashSet::new();

    for root in roots {
        let mut steps = Vec::new();
        let mut cursor = Some(root.id);

        while let Some(id) = cursor {
            // A fact belongs to exactly one chain, so a repeat means a cycle
            // (hand-edited frontmatter). Stop rather than loop forever.
            if !visited.insert(id) {
                break;
            }
            let Some(fact) = by_id.get(&id) else {
                break;
            };
            steps.push(step_of(fact));
            cursor = successors.get(&id).copied();
        }

        if !steps.is_empty() && (include_singletons || steps.len() > 1) {
            chains.push(Chain { steps });
        }
    }

    chains.sort_by(|a, b| {
        let a_time = a.tip().map(|s| s.revised_at);
        let b_time = b.tip().map(|s| s.revised_at);
        b_time.cmp(&a_time)
    });

    chains
}

/// The chain containing `id`, if `facts` describes one.
pub fn chain_for(facts: &[Fact], id: &FactId) -> Option<Chain> {
    build_chains(facts, true)
        .into_iter()
        .find(|chain| chain.steps.iter().any(|step| step.id == *id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{FactType, Scope};

    fn fact(title: &str) -> Fact {
        Fact::new(
            Scope::Project("demo".to_string()),
            FactType::Decision,
            title.to_string(),
            "body".to_string(),
        )
    }

    /// Builds `a -> b -> c` where each revision supersedes the previous one.
    fn linear_chain() -> Vec<Fact> {
        let mut a = fact("v1");
        let mut b = fact("v2");
        let mut c = fact("v3");
        a.status = FactStatus::Deprecated;
        a.validity.until = Some(Utc::now());
        b.status = FactStatus::Deprecated;
        b.validity.until = Some(Utc::now());
        b.supersedes = Some(a.id);
        c.supersedes = Some(b.id);
        vec![a, b, c]
    }

    #[test]
    fn test_linear_chain_is_rebuilt_oldest_first() {
        let facts = linear_chain();
        let chains = build_chains(&facts, false);

        assert_eq!(chains.len(), 1);
        let chain = &chains[0];
        assert_eq!(chain.steps.len(), 3);
        assert_eq!(chain.steps[0].title, "v1");
        assert_eq!(chain.steps[1].title, "v2");
        assert_eq!(chain.steps[2].title, "v3");
        assert!(chain.has_history());
    }

    #[test]
    fn test_tip_is_latest_and_active_is_the_stable_one() {
        let facts = linear_chain();
        let chain = &build_chains(&facts, false)[0];

        assert_eq!(chain.tip().map(|s| s.title.as_str()), Some("v3"));
        assert_eq!(chain.active().map(|s| s.title.as_str()), Some("v3"));
    }

    #[test]
    fn test_unrelated_facts_are_omitted_unless_requested() {
        let mut facts = linear_chain();
        facts.push(fact("unrelated"));

        assert_eq!(build_chains(&facts, false).len(), 1);
        assert_eq!(build_chains(&facts, true).len(), 2);
    }

    #[test]
    fn test_orphaned_revision_is_treated_as_a_root() {
        // A revision whose predecessor is absent (e.g. filtered out by a scope or
        // type query) must become its own root rather than being dropped from the
        // walk or mis-linked to an unrelated fact.
        let mut orphan = fact("v2");
        orphan.supersedes = Some(FactId::new());
        let facts = vec![orphan];

        // A one-step chain is omitted by default, keeping timeline output quiet...
        assert!(build_chains(&facts, false).is_empty());

        // ...but the orphan is still recoverable as a one-step chain.
        let chains = build_chains(&facts, true);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].steps.len(), 1);
        assert_eq!(chains[0].steps[0].title, "v2");
        assert!(!chains[0].has_history());
    }

    #[test]
    fn test_chain_for_finds_the_right_chain() {
        let mut facts = linear_chain();
        facts.push(fact("other"));
        let target = facts[1].id;

        let chain = chain_for(&facts, &target).expect("chain should be found");
        assert_eq!(chain.steps.len(), 3);
        assert!(chain.steps.iter().any(|step| step.id == target));
    }

    #[test]
    fn test_cycle_does_not_hang() {
        // Hand-edited frontmatter could describe a cycle; the walk must stop.
        let mut a = fact("a");
        let mut b = fact("b");
        a.supersedes = Some(b.id);
        b.supersedes = Some(a.id);

        let chains = build_chains(&[a, b], true);
        assert!(chains.iter().all(|chain| chain.steps.len() <= 2));
    }
}
