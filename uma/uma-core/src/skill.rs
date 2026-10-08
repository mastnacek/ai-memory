//! Skill invocation templates: placeholder discovery and expansion.
//!
//! A `skill` fact carries a `template` string with `{{name}}` placeholders; this
//! module resolves those placeholders into concrete text.
//!
//! **It never executes anything.** Expansion is a pure string operation whose
//! only output is text. Running that text is the caller's decision and is
//! governed by the caller's own approval — a harness's shell tool, or a human at
//! a terminal. Keeping execution out of this crate is deliberate: a memory store
//! that can run shell commands is a code-execution surface, and it would bypass
//! whatever approval the surrounding harness already applies to commands.

use std::collections::HashMap;

use anyhow::{bail, Result};

/// Result of resolving a template against a set of variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    /// The template with every supplied placeholder substituted.
    pub output: String,
    /// Placeholders with no supplied value; they are left intact in `output`
    /// so the omission is visible rather than silently rendering empty.
    pub missing: Vec<String>,
    /// Supplied variables the template never referenced (likely a typo).
    pub unused: Vec<String>,
}

/// Placeholders declared in `template`, in first-appearance order, deduplicated.
pub fn placeholders(template: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            break;
        };
        let name = after[..end].trim();
        if !name.is_empty() && !found.iter().any(|existing| existing == name) {
            found.push(name.to_string());
        }
        rest = &after[end + 2..];
    }

    found
}

/// Substitutes `{{name}}` placeholders with values from `vars`.
///
/// Unknown placeholders are left untouched (visible, not blanked) and reported
/// in `Expansion::missing`. An unterminated `{{` is emitted verbatim; malformed
/// templates degrade to text rather than failing, because the output is only
/// ever displayed or handed to the caller.
pub fn expand(template: &str, vars: &HashMap<String, String>) -> Expansion {
    let declared = placeholders(template);
    let mut output = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            output.push_str(&rest[start..]);
            rest = "";
            break;
        };

        let name = after[..end].trim();
        match vars.get(name) {
            Some(value) => output.push_str(value),
            None => output.push_str(&rest[start..start + 2 + end + 2]),
        }
        rest = &after[end + 2..];
    }
    output.push_str(rest);

    let mut unused: Vec<String> = vars
        .keys()
        .filter(|key| !declared.contains(key))
        .cloned()
        .collect();
    unused.sort();

    Expansion {
        output,
        missing: declared
            .into_iter()
            .filter(|name| !vars.contains_key(name))
            .collect(),
        unused,
    }
}

/// Parses repeated `key=value` assignments into a variable map.
pub fn parse_assignments(pairs: &[String]) -> Result<HashMap<String, String>> {
    let mut vars = HashMap::new();
    for pair in pairs {
        let Some((key, value)) = pair.split_once('=') else {
            bail!("expected key=value, got '{pair}'");
        };
        let key = key.trim();
        if key.is_empty() {
            bail!("empty placeholder name in '{pair}'");
        }
        vars.insert(key.to_string(), value.to_string());
    }
    Ok(vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_placeholders_are_found_in_order_and_deduplicated() {
        let template = "docker build -t {{tag}} -f {{ file }} . && echo {{tag}}";
        assert_eq!(placeholders(template), vec!["tag", "file"]);
    }

    #[test]
    fn test_expand_substitutes_every_placeholder() {
        let result = expand(
            "docker build -t {{tag}} .",
            &vars(&[("tag", "v1")]),
        );
        assert_eq!(result.output, "docker build -t v1 .");
        assert!(result.missing.is_empty());
        assert!(result.unused.is_empty());
    }

    #[test]
    fn test_missing_values_are_reported_and_left_visible() {
        let result = expand("deploy {{app}} to {{env}}", &vars(&[("app", "api")]));
        assert_eq!(result.output, "deploy api to {{env}}");
        assert_eq!(result.missing, vec!["env"]);
    }

    #[test]
    fn test_unused_values_are_reported() {
        let result = expand("build {{tag}}", &vars(&[("tag", "v1"), ("typo", "x")]));
        assert_eq!(result.unused, vec!["typo"]);
    }

    #[test]
    fn test_template_without_placeholders_passes_through() {
        let template = "cargo test --all";
        let result = expand(template, &HashMap::new());
        assert_eq!(result.output, template);
        assert!(result.missing.is_empty());
    }

    #[test]
    fn test_unterminated_placeholder_degrades_to_text() {
        let result = expand("echo {{oops", &HashMap::new());
        assert_eq!(result.output, "echo {{oops");
        assert!(result.missing.is_empty());
    }

    #[test]
    fn test_parse_assignments() {
        let parsed = parse_assignments(&["tag=v1".to_string(), "env=prod".to_string()]).unwrap();
        assert_eq!(parsed.get("tag").map(String::as_str), Some("v1"));
        assert_eq!(parsed.get("env").map(String::as_str), Some("prod"));

        assert!(parse_assignments(&["no-equals-sign".to_string()]).is_err());
        assert!(parse_assignments(&["=value".to_string()]).is_err());
    }
}
