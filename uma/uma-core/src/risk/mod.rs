//! File risk scoring: the deterministic "pain score".
//!
//! Ports the *deterministic* half of Proposal 04 (docs/proposals/04): a file's
//! history of corrections, reverts, and churn becomes a 0–100 score with a
//! three-band policy. It is deliberately offline and reproducible — `cargo
//! test` exercises the exact formula production uses — and it is **advisory**:
//! it feeds warnings and prompts, never blocks by itself. Probabilistic
//! judgment (Jev) may refine it later, but the floor must not depend on a
//! network model.

use serde::Serialize;

/// Policy band derived from the score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Band {
    /// 0–20: proceed normally.
    Low,
    /// 21–60: run the affected tests after the edit.
    Medium,
    /// 61–100: test-first discipline — propose the failing test before the fix.
    Critical,
}

impl Band {
    pub fn as_str(self) -> &'static str {
        match self {
            Band::Low => "low",
            Band::Medium => "medium",
            Band::Critical => "critical",
        }
    }

    fn from_score(score: u32) -> Band {
        if score >= 61 {
            Band::Critical
        } else if score >= 21 {
            Band::Medium
        } else {
            Band::Low
        }
    }
}

/// The scored file, with every input kept visible for the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PainScore {
    /// 0–100, clamped.
    pub score: u32,
    pub band: Band,
    /// Memory corrections mentioning this file (how often it hurt before).
    pub correction_mentions: usize,
    /// Git reverts of this file (how often work was undone).
    pub reverts: usize,
    /// Commits touching the file inside the observation window (churn).
    pub churn: usize,
}

/// Per-signal weights and caps: a cap keeps any single signal from owning the
/// score, so a heavily-churned file that never broke is not punished into the
/// critical band by churn alone.
const MENTION_WEIGHT: u32 = 15;
const MENTION_CAP: u32 = 45;
const REVERT_WEIGHT: u32 = 20;
const REVERT_CAP: u32 = 60;
const CHURN_WEIGHT: u32 = 2;
const CHURN_CAP: u32 = 20;

/// Computes the pain score from already-gathered signals.
///
/// The kernel takes plain numbers, not a git handle: gathering history is the
/// CLI slice's job, scoring is pure math both sides can test.
pub fn compute(correction_mentions: usize, reverts: usize, churn: usize) -> PainScore {
    let mention_points = (correction_mentions as u32 * MENTION_WEIGHT).min(MENTION_CAP);
    let revert_points = (reverts as u32 * REVERT_WEIGHT).min(REVERT_CAP);
    let churn_points = (churn as u32 * CHURN_WEIGHT).min(CHURN_CAP);

    let score = (mention_points + revert_points + churn_points).min(100);
    PainScore {
        score,
        band: Band::from_score(score),
        correction_mentions,
        reverts,
        churn,
    }
}

/// The operator-facing guidance for a band.
pub fn guidance(band: Band) -> &'static str {
    match band {
        Band::Low => "Normal edits allowed.",
        Band::Medium => "Run the affected tests after editing.",
        Band::Critical => "Test-first: propose the failing test before changing this file.",
    }
}

#[cfg(test)]
mod tests;