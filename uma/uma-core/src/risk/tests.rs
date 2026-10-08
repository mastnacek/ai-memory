//! Pain-score tests: the formula and its bands.

use super::*;

#[test]
fn test_clean_file_is_low() {
    let score = compute(0, 0, 0);
    assert_eq!(score.score, 0);
    assert_eq!(score.band, Band::Low);
}

#[test]
fn test_weights_compose_and_cap() {
    // 2 mentions (30) + 1 revert (20) = 50 → medium.
    let score = compute(2, 1, 0);
    assert_eq!(score.score, 50);
    assert_eq!(score.band, Band::Medium);

    // A single signal cannot exceed its cap: 10 mentions = 150 raw, capped 45.
    let score = compute(10, 0, 0);
    assert_eq!(score.score, 45);
    assert_eq!(score.correction_mentions, 10);
}

#[test]
fn test_churn_alone_never_reaches_critical() {
    // Churn cap (20) stays below the medium boundary (21): a heavily-touched
    // file that never broke and was never reverted is still low-risk.
    let score = compute(0, 0, 50);
    assert_eq!(score.score, 20);
    assert_eq!(score.band, Band::Low);
}

#[test]
fn test_history_of_pain_reaches_critical() {
    // 3 mentions (45) + 3 reverts (60, capped) = 105 → clamped 100, critical.
    let score = compute(3, 3, 0);
    assert_eq!(score.score, 100);
    assert_eq!(score.band, Band::Critical);
    assert_eq!(guidance(score.band), "Test-first: propose the failing test before changing this file.");
}

#[test]
fn test_band_boundaries() {
    // Churn alone can never leave the low band: 10*2 = 20 < 21, and the
    // 11-commit case is already capped back to 20.
    assert_eq!(compute(0, 0, 10).band, Band::Low);
    assert_eq!(compute(0, 0, 11).band, Band::Low);
    // The medium boundary needs mixed signals: 1 mention (15) + 3 churn (6).
    assert_eq!(compute(1, 0, 3).band, Band::Medium); // 21
    let critical = compute(3, 1, 0); // 45 + 20 = 65
    assert_eq!(critical.band, Band::Critical);
}
