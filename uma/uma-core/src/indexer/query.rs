//! FTS5 query construction and scope parsing.

use crate::domain::{Scope};

/// Builds a prefix-match FTS5 query from free-form user input.
pub fn build_fts_query(query: &str) -> String {
    let tokens: Vec<&str> = query.split_whitespace().collect();
    if tokens.is_empty() {
        return query.to_string();
    }
    tokens
        .iter()
        .map(|t| {
            let clean = t.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if clean.is_empty() {
                format!("\"{}\"", t)
            } else {
                format!("{}*", clean)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Builds a quoted FTS5 query safe against syntax errors from special characters.
pub fn build_safe_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_scope_str(s: &str) -> Scope {
    if s == "global" {
        Scope::Global
    } else if let Some(name) = s.strip_prefix("project:") {
        Scope::Project(name.to_string())
    } else {
        Scope::Project(s.to_string())
    }
}

