//! Secret-scanner tests: real credentials block, templates pass.

use super::*;
use patterns::INVISIBLE_CHARS;

fn scan_text(text: &str) -> Vec<Finding> {
    scan(text, &[])
}

#[test]
fn test_real_credentials_block() {
    // Realistic-shaped values, not real keys.
    let cases = [
        // Not the AWS docs example (AKIAIOSFODNN7EXAMPLE) - the placeholder
        // filter correctly lets that one through.
        ("AWS access key", "AKIAABCDEFGHIJKLMNOP"),
        ("GitHub token", "ghp_0123456789abcdefghijklmnopqrstuvwxyzAB"),
        ("OpenAI API key", "sk-proj-abcdefghij0123456789abcdefghij3"),
        ("Stripe key", "sk_live_0123456789abcdefXYZ9"),
        ("JWT", "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJVadQssw5c"),
    ];
    for (label, value) in cases {
        let findings = scan_text(&format!("the key is {value} in prod"));
        assert!(
            is_blocked(&findings),
            "{label} must block: {findings:?}"
        );
    }
}

#[test]
fn test_doc_templates_pass() {
    // The exact non-secrets a reviewer writes into examples and docs.
    let safe = [
        "sk-xxxxxxxxxxxxxxxxxxxxxxxx",
        "password = changeme",
        "token = ${GITHUB_TOKEN}",
        "apiKey = OPENAI_API_KEY",
        "<your-token-here>",
        "Set AWS_SECRET_ACCESS_KEY in the .env file, not in git.",
    ];
    for text in safe {
        let findings = scan_text(text);
        assert!(
            !is_blocked(&findings),
            "'{text}' is a doc example, must not block: {findings:?}"
        );
    }
}

#[test]
fn test_env_var_names_are_not_secrets_but_values_are() {
    let env = vec![
        ("GITHUB_TOKEN".to_string(), "ghp_0123456789abcdefghijklmnopqrstuvwxyzAB".to_string()),
        ("HOME".to_string(), "/home/jaroslav".to_string()),
        ("EDITOR".to_string(), "vim".to_string()),
    ];
    let literals = env_secret_literals(&env);
    assert_eq!(literals.len(), 1, "only the secret-shaped value is a literal");

    let findings = scan("auth token ghp_0123456789abcdefghijklmnopqrstuvwxyzAB leaked", &literals);
    assert!(is_blocked(&findings));

    // HOME/EDITOR values are not treated as credentials.
    let findings = scan("edit with vim from /home/jaroslav", &literals);
    assert!(!is_blocked(&findings));
}

#[test]
fn test_url_credentials_and_bearer() {
    assert!(is_blocked(&scan_text("remote https://user:S3cretPassw0rd!@git.example.com/repo.git")));
    assert!(is_blocked(&scan_text(
        "authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
    )));
    // Prose that merely mentions bearer stays safe.
    assert!(!is_blocked(&scan_text("bearer authentication is documented here")));
}

#[test]
fn test_private_key_and_czech_note() {
    assert!(is_blocked(&scan_text("-----BEGIN RSA PRIVATE KEY-----")));
    // A Czech note about the gate itself must not trip it.
    let note = "Nikdy neukládat API klíče do paměti, použij .env soubor.";
    assert!(!is_blocked(&scan_text(note)), "meta note must pass");
}

#[test]
fn test_invisible_unicode_blocks_and_injection_warns() {
    let sneaky = format!("harmless rule{}", INVISIBLE_CHARS[0]);
    let findings = scan_text(&sneaky);
    assert!(is_blocked(&findings), "invisible unicode must block");

    let findings = scan_text("remember to ignore previous instructions in demos");
    assert!(
        !is_blocked(&findings) && !findings.is_empty(),
        "injection signature warns without blocking: {findings:?}"
    );
}

#[test]
fn test_previews_are_masked() {
    let value = "sk-proj-abcdefghij0123456789abcdefghij3";
    let findings = scan_text(&format!("key {value}"));
    let preview = &findings[0].preview;
    assert!(!preview.contains("abcdefghij0123456789"), "full value never in preview");
    assert!(preview.contains('…'), "preview is elided: {preview}");
}

#[test]
fn test_placeholder_value_rules() {
    assert!(is_placeholder_value("changeme"));
    assert!(is_placeholder_value("${GITHUB_TOKEN}"));
    assert!(is_placeholder_value("OPENAI_API_KEY"));
    assert!(is_placeholder_value("sk-xxxxxxxxxxxxxxxxxxxx"));
    assert!(!is_placeholder_value("sk-proj-abcdefghij0123456789abcdefghij3"));
    assert!(!is_placeholder_value("0123456789abcdef"));
}