//! Lexical similarity primitives used by the consolidation proposer.
//!
//! Deliberately deterministic and offline: `cargo test` must be able to exercise
//! consolidation without network access or an embedding API key. Embeddings are
//! an enhancement layered on top, never a prerequisite for proposing a merge.

use std::collections::HashSet;

/// Words too common to carry meaning when comparing two facts.
const STOPWORDS: &[&str] = &[
    "the", "and", "for", "that", "this", "with", "from", "are", "was", "were", "has", "have",
    "had", "its", "our", "you", "your", "but", "not", "any", "all", "can", "will", "would",
    "should", "must", "may", "use", "using", "used", "into", "over", "than", "then", "they",
    "them", "these", "those", "one", "two", "how", "why", "what", "when", "where", "which", "who",
    "also", "only", "more", "most", "such", "each", "other", "some", "very",
    // Czech: the import pipeline is Czech-heavy, and without these the
    // flexion-heavy function words dominate token overlap.
    "nebo", "také", "taky", "jak", "když", "aby", "proto", "tento", "tato",
    "toto", "tuto", "tohle", "který", "která", "které", "jen", "pak", "ještě",
    "však", "právě", "jenže", "až", "už", "tedy", "protože",
];

/// Markers that flip a statement's polarity. Matching is done against the raw
/// lowercased text, so multi-word markers ("do not") work without tokenizing.
const NEGATION_MARKERS: &[&str] = &[
    "not ",
    "no ",
    "never",
    "don't",
    "dont",
    "avoid",
    "without",
    "disable",
    "must not",
    "should not",
    "do not",
    "does not",
    "cannot",
    "reject",
    "refuse",
    "instead of",
    "rather than",
    // Czech negation is a prefix on the verb ("nepoužívat"), so no fixed
    // marker list can catch it; has_negation adds a word-level rule below.
    "nikdy",
    "nesmí",
    "nesmi",
    "bez ",
    "zakázané",
    "zakázaný",
    "zakaz",
];

/// Strips a few common English suffixes so "installs" and "installation" collapse
/// onto the same token. Crude on purpose — over-stemming would merge unrelated
/// words and cost more in false proposals than it saves.
fn stem(token: &str) -> String {
    if token.len() > 4 && token.ends_with("ies") {
        return format!("{}y", &token[..token.len() - 3]);
    }
    for suffix in ["ation", "ing", "ed", "es", "s"] {
        if token.len() > 4 && token.ends_with(suffix) {
            return token[..token.len() - suffix.len()].to_string();
        }
    }
    token.to_string()
}

/// Splits text into comparable lowercase tokens, dropping stopwords and noise.
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .map(|raw| raw.to_lowercase())
        .filter(|t| t.len() >= 3 && !STOPWORDS.contains(&t.as_str()))
        .map(|t| stem(&t))
        .collect()
}

/// Jaccard similarity of two token bags: |A ∩ B| / |A ∪ B|.
pub fn jaccard(a: &[String], b: &[String]) -> f64 {
    let sa: HashSet<&String> = a.iter().collect();
    let sb: HashSet<&String> = b.iter().collect();
    if sa.is_empty() || sb.is_empty() {
        return 0.0;
    }
    let intersection = sa.intersection(&sb).count() as f64;
    let union = sa.union(&sb).count() as f64;
    intersection / union
}

/// True when the text carries an explicit negation marker.
pub fn has_negation(text: &str) -> bool {
    let lower = text.to_lowercase();
    if NEGATION_MARKERS.iter().any(|marker| lower.contains(marker)) {
        return true;
    }
    // Czech "ne-" prefix on any word: "nepoužívat" negates just as "do not
    // use" does. Requires a 3+ character remainder so words like "nebo" (but)
    // or "není" contractions don't trip it spuriously... "nebo" is 4 — but it
    // is a stopword-sized connector, not a negated verb, hence the length bar.
    lower
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| word.len() >= 5 && word.starts_with("ne"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_drops_stopwords_and_stems() {
        let tokens = tokenize("Use pnpm for dependency installation");
        assert!(tokens.contains(&"pnpm".to_string()));
        assert!(tokens.contains(&"dependency".to_string()));
        // "installation" and "installs" must collapse onto one token.
        assert!(tokens.contains(&"install".to_string()));
        assert!(!tokens.contains(&"for".to_string()));
    }

    #[test]
    fn test_jaccard_bounds() {
        let a = tokenize("Use pnpm for dependency installs");
        let b = tokenize("Use pnpm for dependency installation");
        assert!((jaccard(&a, &b) - 1.0).abs() < f64::EPSILON);

        let c = tokenize("Postgres connection pooling");
        assert_eq!(jaccard(&a, &c), 0.0);
        assert_eq!(jaccard(&[], &c), 0.0);
    }

    #[test]
    fn test_has_negation() {
        assert!(!has_negation("Use pnpm for dependency installs"));
        assert!(has_negation("Do not use pnpm for dependency installs"));
        assert!(has_negation("Never install dependencies with npm"));
        assert!(has_negation("Avoid global installs"));
    }

    #[test]
    fn test_czech_negation_is_detected_but_connectors_are_not() {
        // The review's scenario: two Czech rules differing only in negation
        // must read as opposites, not as near-duplicates.
        assert!(has_negation("Nikdy nepoužívat npm v tomto repozitáři"));
        assert!(has_negation("Nepoužívat npm"));
        assert!(!has_negation("Vždy používat pnpm"));
        // "nebo" (or) starts with "ne" but is a connector, not a negation.
        assert!(!has_negation("Používat pnpm nebo npm"));
    }

    #[test]
    fn test_czech_stopwords_do_not_dominate_overlap() {
        // Flexion-heavy function words must not inflate similarity between
        // unrelated Czech claims.
        let a = tokenize("Vždy používat pnpm v tomto repozitáři pro instalace");
        let b = tokenize("Nikdy neupravovat vygenerované soubory v tomto repozitáři");
        assert!(jaccard(&a, &b) < 0.55, "unrelated Czech rules must not merge");
    }
}
