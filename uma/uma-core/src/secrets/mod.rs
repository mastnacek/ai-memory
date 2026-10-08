//! Secret scanning: credentials must never enter memory.
//!
//! Ported from the `@pify/memory` secret gate (which credits
//! `pi-hermes-memory` for the idea), with one policy change for UMA: this is
//! a *consent-gated* system, so the scanner distinguishes **block** findings
//! (credentials, invisible characters — nothing legitimate looks like this)
//! from **warning** findings (injection signatures, which fire on legitimate
//! notes *about* injection). The CLI write path refuses on block findings;
//! warnings are surfaced, not enforced.
//!
//! Deterministic, offline, zero network: `cargo test` runs the gate exactly
//! as production does.

use std::sync::OnceLock;

use serde::Serialize;

pub mod patterns;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Nothing legitimate matches this — the write must be refused.
    Block,
    /// A hazard signature worth surfacing; the operator decides.
    Warning,
}

/// One scanner finding. `preview` is always masked — the scanner must never
/// log or print a full credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub label: String,
    pub severity: Severity,
    pub preview: String,
}

static COMPILED: OnceLock<patterns::Compiled> = OnceLock::new();

/// The single panic point in this module: a built-in literal pattern failing
/// to compile is a review-time bug, not a runtime condition.
fn compiled() -> &'static patterns::Compiled {
    COMPILED.get_or_init(|| patterns::compile().expect("built-in scanner patterns must compile"))
}

/// Is this matched value plainly a placeholder rather than a real secret?
///
/// Deliberately narrow: everything here is unambiguously not-a-credential, so
/// a real key (mixed case, high entropy) is never suppressed. Suppression
/// covers templates (`${VAR}`, `{{x}}`), env-var *names* (`SCREAMING_SNAKE`),
/// single repeated characters (`sk-xxxx…`), and placeholder words
/// (`changeme`, `example`, `redacted`).
pub fn is_placeholder_value(raw: &str) -> bool {
    let res = compiled();
    let s = raw.trim().trim_matches(|c| c == '"' || c == '\'');
    if s.is_empty() {
        return true;
    }
    if s.contains("${") || s.contains("{{") {
        return true;
    }
    if s.contains("%s") || s.contains("%d") || s.contains("%v") {
        return true;
    }
    if res.xml_tag.is_match(s) || res.env_ref.is_match(s) || res.screaming_snake.is_match(s) {
        return true;
    }
    // The value body (after any provider prefix) is one repeated character.
    let body: String = res.provider_prefix.replace(s, "").into_owned();
    let mut chars = body.chars();
    if body.chars().count() >= 4
        && chars
            .next()
            .is_some_and(|first| body.chars().all(|c| c == first))
    {
        return true;
    }
    if res.placeholder_words.is_match(s) {
        return true;
    }
    false
}

/// Variable names that hold credentials by convention; the VALUE of such a
/// variable is a secret whatever it looks like — an internal API key with no
/// provider prefix passes every shape pattern, yet the exact string sits in
/// the environment.

/// A value that is a plain word is configuration, not a credential — a refusal
/// is not lossless, so precision wins.
fn looks_like_word(value: &str) -> bool {
    let lower = value.to_lowercase();
    matches!(
        lower.as_str(),
        "password"
            | "changeme"
            | "disabled"
            | "enabled"
            | "true"
            | "false"
            | "localhost"
            | "default"
            | "example"
            | "secret"
            | "token"
            | "undefined"
            | "null"
            | "none"
            | "required"
            | "optional"
            | "development"
            | "production"
    ) || compiled().word_value.is_match(value)
}

/// The literal secret values this process knows: values of secret-shaped
/// variables (8+ chars, not a word, not a placeholder) and the password of any
/// `scheme://user:pass@host` value under any name. Never persisted or logged;
/// only ever compared against the text.
pub fn env_secret_literals(env: &[(String, String)]) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();
    let (name_re, url_re) = (&compiled().secret_name, &compiled().url_creds);
    for (name, raw) in env {
        let value = raw.trim();
        if value.is_empty() {
            continue;
        }
        if let Some(url) = url_re.captures(value) {
            let password = &url[1];
            if password.len() >= 8 && !looks_like_word(password) && !is_placeholder_value(password)
            {
                out.insert(password.to_string());
            }
        }
        if !name_re.is_match(name) {
            continue;
        }
        if value.len() >= 8 && !looks_like_word(value) && !is_placeholder_value(value) {
            out.insert(value.to_string());
        }
    }
    out.into_iter().collect()
}

/// Masked preview: never prints a full credential.
fn preview_of(raw: &str) -> String {
    if raw.chars().count() <= 12 {
        let head: String = raw.chars().take(4).collect();
        format!("{head}…")
    } else {
        let chars: Vec<char> = raw.chars().collect();
        let head: String = chars[..8].iter().collect();
        let tail: String = chars[chars.len() - 2..].iter().collect();
        format!("{head}…{tail}")
    }
}

/// Scans text for credential leaks (block) and injection signatures (warning).
/// `env_literals` are the process's known secret values; an exact literal hit
/// is a credential by definition, shape or no shape.
pub fn scan(text: &str, env_literals: &[String]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (re, label, value_group) in &compiled().patterns {
        let Some(m) = re.captures(text) else {
            continue;
        };
        let whole = m.get(0).map(|m| m.as_str()).unwrap_or_default();
        let value = match value_group {
            Some(g) => m.get(*g).map(|m| m.as_str()).unwrap_or(whole),
            None => whole,
        };
        if is_placeholder_value(value) {
            continue;
        }
        let severity = if patterns::SECRET_PATTERNS.iter().any(|p| p.label == *label) {
            Severity::Block
        } else {
            Severity::Warning
        };
        findings.push(Finding {
            label: (*label).to_string(),
            severity,
            preview: preview_of(whole),
        });
    }
    for literal in env_literals {
        if text.contains(literal.as_str()) {
            findings.push(Finding {
                label: "known credential from the environment".to_string(),
                severity: Severity::Block,
                preview: preview_of(literal),
            });
            break;
        }
    }
    for ch in text.chars() {
        if patterns::INVISIBLE_CHARS.contains(&ch) {
            findings.push(Finding {
                label: format!(
                    "invisible unicode character U+{:04X} (possible injection)",
                    ch as u32
                ),
                severity: Severity::Block,
                preview: format!("U+{:04X}", ch as u32),
            });
            break;
        }
    }
    findings
}

/// True when the text must not be saved.
pub fn is_blocked(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.severity == Severity::Block)
}

#[cfg(test)]
mod tests;
