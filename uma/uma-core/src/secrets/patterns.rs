//! Provider credential patterns and injection threat signatures.
//!
//! Ported from the betterleaks-grade ruleset in `@pify/memory` (`secrets.ts`),
//! which credits `pi-hermes-memory` for the idea, plus hermes's invisible
//! unicode set. Kept as plain data so a new provider pattern is a one-line
//! addition reviewed like any table, not logic.

use regex::Regex;

/// One credential shape. `value_group` selects the capture holding the secret
/// value; `None` means the whole match is the value.
pub struct Pattern {
    pub label: &'static str,
    pub source: &'static str,
    pub value_group: Option<usize>,
}

macro_rules! pattern {
    ($label:literal, $re:literal) => {
        Pattern {
            label: $label,
            source: $re,
            value_group: None,
        }
    };
    ($label:literal, $re:literal, value_group = $group:literal) => {
        Pattern {
            label: $label,
            source: $re,
            value_group: Some($group),
        }
    };
}

/// High-signal, anchored provider patterns — a match is a credential unless it
/// is a placeholder (see `is_placeholder_value`).
pub const SECRET_PATTERNS: &[Pattern] = &[
    pattern!("AWS access key", r"\b(AKIA|ASIA)[0-9A-Z]{16}\b"),
    pattern!("GitHub token", r"\b(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,}\b"),
    pattern!("GitHub fine-grained token", r"\bgithub_pat_[A-Za-z0-9_]{22,}\b"),
    pattern!("GitLab token", r"\bglpat-[A-Za-z0-9_-]{20,}\b"),
    pattern!("Slack token", r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b"),
    pattern!("Slack webhook", r"\bhttps://hooks\.slack\.com/services/[A-Za-z0-9/_-]{20,}"),
    pattern!("Stripe key", r"\b[rs]k_live_[A-Za-z0-9]{16,}\b"),
    pattern!("SendGrid key", r"\bSG\.[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{16,}\b"),
    pattern!("Google OAuth secret", r"\bGOCSPX-[A-Za-z0-9_-]{20,}\b"),
    pattern!("OpenAI API key", r"\bsk-[A-Za-z0-9_-]{20,}\b"),
    pattern!("Anthropic API key", r"\bsk-ant-[A-Za-z0-9_-]{20,}\b"),
    pattern!("Google API key", r"\bAIza[0-9A-Za-z_-]{35}\b"),
    pattern!("npm token", r"\bnpm_[A-Za-z0-9]{36,}\b"),
    pattern!("private key block", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    pattern!("JWT", r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b"),
    pattern!(
        "assigned credential",
        r#"(?i)\b(api[_-]?key|secret|token|password|passwd)\b\s*[:=]\s*["']?([A-Za-z0-9+/_-]{16,})["']?"#,
        value_group = 2
    ),
    // The shape of a pasted `curl -H` line or captured request. Anchored on
    // the literal "authorization" so prose that merely says "bearer" never
    // fires (the async-fork redaction leak path).
    pattern!(
        "authorization header",
        r#"(?i)\bauthorization\b\s*[:=]\s*["']?(?:bearer|basic|token)\s+(\S{8,})"#,
        value_group = 1
    ),
    // "bearer" plus a 24-char single token is a credential, not a sentence —
    // "bearer of the news" falls short of the length.
    pattern!("bearer token", r"(?i)\bbearer\s+([A-Za-z0-9._~+/-]{24,}={0,2})", value_group = 1),
    // scheme://user:pass@host — git remotes, database URLs, authenticated
    // curl. `://x:y@` never occurs in prose.
    // scheme://user:pass@host - git remotes, database URLs, authenticated
    // curl. The PASSWORD is the captured value, not the whole URL: the
    // placeholder filter must judge the credential, never the domain (an
    // example.com host must not suppress a real password beside it).
    pattern!(
        "credentials in URL",
        r"\b[a-z][a-z0-9+.-]*://[^\s:/?#@]+:([^\s:/?#@]+)@\S+",
        value_group = 1
    ),
];

/// Prompt-injection / exfiltration payloads. WARN severity, not block: these
/// fire on legitimate notes *about* injection, and the operator modal is the
/// last line of defense — the scanner's job is to make the hazard visible.
pub const THREAT_PATTERNS: &[Pattern] = &[
    pattern!("prompt injection", r"(?i)ignore\s+(previous|all|above|prior)\s+instructions"),
    pattern!("role hijack", r"(?i)you\s+are\s+now\s+"),
    pattern!("deception", r"(?i)do\s+not\s+tell\s+the\s+user"),
    pattern!("prompt override", r"(?i)system\s+prompt\s+override"),
    pattern!("disregard rules", r"(?i)disregard\s+(your|all|any)\s+(instructions|rules|guidelines)"),
    pattern!("restriction bypass", r"(?i)act\s+as\s+(if|though)\s+you\s+(have\s+no|don'?t\s+have)\s+(restrictions|limits|rules)"),
    pattern!("credential exfiltration", r"(?i)(curl|wget)\s+[^\n]*\$\{?\w*(KEY|TOKEN|SECRET|PASSWORD|CREDENTIAL|API)"),
    pattern!("secret file read", r"(?i)cat\s+[^\n]*(\.env|credentials|\.netrc|\.pgpass|\.npmrc|\.pypirc)"),
    pattern!("ssh access", r"(?i)\$HOME/\.ssh|~/\.ssh|authorized_keys"),
];

/// Unicode codepoints that are invisible in editors and terminals — never
/// legitimate in a memory fact, always a hiding place for injected text.
pub const INVISIBLE_CHARS: &[char] = &[
    '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}', '\u{202a}', '\u{202b}',
    '\u{202c}', '\u{202d}', '\u{202e}',
];

/// Every regex the scanner needs, built in one fallible place. A bad literal
/// would be a review-time bug, so the single lazy initializer panics once;
/// nothing else in the module is allowed to construct or panic.
pub struct Compiled {
    pub patterns: Vec<(Regex, &'static str, Option<usize>)>,
    pub xml_tag: Regex,
    pub env_ref: Regex,
    pub screaming_snake: Regex,
    pub provider_prefix: Regex,
    pub placeholder_words: Regex,
    pub word_value: Regex,
    pub secret_name: Regex,
    pub url_creds: Regex,
}

pub fn compile() -> Result<Compiled, regex::Error> {
    let mut compiled = Vec::new();
    for pattern in SECRET_PATTERNS.iter().chain(THREAT_PATTERNS.iter()) {
        compiled.push((Regex::new(pattern.source)?, pattern.label, pattern.value_group));
    }
    Ok(Compiled {
        patterns: compiled,
        xml_tag: Regex::new(r"<[A-Za-z_][A-Za-z0-9_]*>")?,
        env_ref: Regex::new(r"^\$[A-Za-z_][A-Za-z0-9_]*$")?,
        screaming_snake: Regex::new(r"^[A-Z][A-Z0-9]*(?:_[A-Z0-9]+)+$")?,
        provider_prefix: Regex::new(
            r"^(?:[rs]k|ghp|gho|ghu|ghs|ghr|glpat|npm|xox[a-z]|sk-ant|GOCSPX|SG|AIza|AKIA|ASIA)[-_.]?",
        )?,
        placeholder_words: Regex::new(
            r#"(?i)change[_-]?me|example|placeholder|redacted|dummy|sample|your[-_ ]?(key|token|secret|password)|todo"#,
        )?,
        word_value: Regex::new(r"^[a-z]{1,15}$")?,
        secret_name: Regex::new(r"(?i)(KEY|SECRET|TOKEN|PASSWORD|PASSWD|AUTH|CREDENTIAL|PRIVATE|OAUTH)")?,
        url_creds: Regex::new(r"(?i)^[a-z][a-z0-9+.-]*://[^\s:/?#@]+:([^\s:/?#@]+)@")?,
    })
}