//! MCP `tools/call` dispatch and the individual tool handlers.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde_json::Value;
use uma_core::consolidate::{analyze, AnalyzeOptions};
use uma_core::domain::{Fact, FactId, FactType, Scope};
use uma_core::search::SearchMode;
use uma_core::store::Store;

use crate::shared::format::{render_fact, render_fact_summary, status_suffix};
use crate::shared::scope::resolve_scope;
use crate::shared::store_helper::get_store;

use super::catalog::{is_mutating, WRITE_REFUSAL};

/// Dispatches one `tools/call`. `Err` becomes an `isError` tool result.
pub fn call(name: &str, args: &Value, allow_writes: bool) -> Result<String, String> {
    // Enforced here, not only by omission from `tools/list`: a client that
    // guesses a mutating tool name must still be refused, otherwise the launch
    // flag would be advisory rather than binding.
    if is_mutating(name) && !allow_writes {
        return Err(WRITE_REFUSAL.to_string());
    }

    match name {
        "uma_read" => read(args),
        "uma_list" => list(args),
        "uma_search" => search(args),
        "uma_consolidate" => consolidate(args),
        "uma_write" => write(args),
        "uma_supersede" => supersede(args),
        other => Err(format!("Unknown tool '{other}'.")),
    }
}

// ---------------------------------------------------------------- arguments

pub(super) fn str_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn bool_arg(args: &Value, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn usize_arg(args: &Value, key: &str, fallback: usize) -> usize {
    args.get(key)
        .and_then(Value::as_u64)
        .map(|n| n as usize)
        .unwrap_or(fallback)
}

fn f64_arg(args: &Value, key: &str) -> Option<f64> {
    args.get(key).and_then(Value::as_f64)
}

fn tags_arg(args: &Value) -> Vec<String> {
    args.get("tags")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(|t| t.trim().to_lowercase())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn scope_arg(args: &Value) -> Result<Scope, String> {
    resolve_scope(str_arg(args, "scope")).map_err(|err| err.to_string())
}

pub(super) fn type_arg(args: &Value) -> Result<Option<FactType>, String> {
    str_arg(args, "type")
        .map(|raw| FactType::from_str(&raw).map_err(|err| err.to_string()))
        .transpose()
}

fn require(args: &Value, key: &str) -> Result<String, String> {
    str_arg(args, key).ok_or_else(|| format!("Missing required argument '{key}'."))
}

fn require_id(args: &Value, key: &str) -> Result<FactId, String> {
    let raw = require(args, key)?;
    FactId::from_str(&raw).map_err(|_| format!("'{raw}' is not a valid fact ULID."))
}

// ---------------------------------------------------------------- handlers

fn read(args: &Value) -> Result<String, String> {
    let id = require_id(args, "id")?;
    let fact = Store::find_by_id(&id).map_err(|err| err.to_string())?;
    Ok(render_fact(&fact))
}

fn list(args: &Value) -> Result<String, String> {
    let scope = scope_arg(args)?;
    let fact_type = type_arg(args)?;
    let store = get_store(&scope).map_err(|err| err.to_string())?;
    let facts = store
        .list(&scope, fact_type.as_ref())
        .map_err(|err| err.to_string())?;

    let stored = facts.len();
    let now = Utc::now();
    let visible: Vec<&Fact> = if bool_arg(args, "includeDeprecated") {
        facts.iter().collect()
    } else {
        facts.iter().filter(|f| f.is_active_at(now)).collect()
    };

    if visible.is_empty() {
        return Ok(if stored > 0 {
            format!("No active facts in {scope} ({stored} hidden as deprecated).")
        } else {
            format!("No facts found in {scope}.")
        });
    }

    let mut out = String::new();
    for fact in visible {
        out.push_str(&render_fact_summary(fact));
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}

fn search(args: &Value) -> Result<String, String> {
    let query = require(args, "query")?;
    let mode = str_arg(args, "mode")
        .map(|m| SearchMode::from_str(&m).unwrap_or(SearchMode::Hybrid))
        .unwrap_or(SearchMode::Hybrid);

    let scope = str_arg(args, "scope")
        .map(|s| resolve_scope(Some(s)).map_err(|err| err.to_string()))
        .transpose()?;
    let fact_type = type_arg(args)?;
    let as_of = str_arg(args, "asOf")
        .map(|raw| {
            DateTime::parse_from_rfc3339(&raw)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|err| format!("Invalid asOf timestamp: {err}"))
        })
        .transpose()?;

    let outcome = Store::search_all(
        &query,
        scope.as_ref(),
        fact_type.as_ref(),
        mode,
        bool_arg(args, "includeDeprecated"),
        as_of,
        usize_arg(args, "limit", 10),
    )
    .map_err(|err| err.to_string())?;

    // A degraded mode is surfaced first, so a client cannot mistake BM25
    // ranking for the hybrid ranking it asked for.
    let note = outcome.note();
    let hits = &outcome.hits;

    if hits.is_empty() {
        let mut out = format!("No facts matching '{query}' (mode: {mode:?}).");
        if let Some(note) = note {
            out.push_str(&format!("\n! {note}"));
        }
        return Ok(out);
    }

    let mut out = String::new();
    if let Some(note) = note {
        out.push_str(&format!("! {note}\n\n"));
    }
    out.push_str(&format!(
        "Found {} match(es) [mode: {mode:?}]:\n",
        hits.len()
    ));
    for (idx, hit) in hits.iter().enumerate() {
        out.push_str(&format!(
            "{}. {} [{}]{} (score {:.3})\n   ID: {}\n",
            idx + 1,
            hit.title,
            hit.fact_type,
            status_suffix(&hit.status),
            hit.score,
            hit.id
        ));
    }
    Ok(out.trim_end().to_string())
}

fn consolidate(args: &Value) -> Result<String, String> {
    let scope = scope_arg(args)?;
    let fact_type = type_arg(args)?;
    let store = get_store(&scope).map_err(|err| err.to_string())?;
    let facts = store
        .list(&scope, fact_type.as_ref())
        .map_err(|err| err.to_string())?;

    let opts = AnalyzeOptions {
        threshold: f64_arg(args, "threshold").unwrap_or(0.55).clamp(0.0, 1.0),
        include_deprecated: bool_arg(args, "includeDeprecated"),
        ..Default::default()
    };
    let report = analyze(&facts, &opts);
    serde_json::to_string_pretty(&report).map_err(|err| err.to_string())
}

fn write(args: &Value) -> Result<String, String> {
    let title = require(args, "title")?;
    let body = require(args, "body")?;
    let fact_type = type_arg(args)?.unwrap_or(FactType::Note);
    let scope = scope_arg(args)?;
    let store = get_store(&scope).map_err(|err| err.to_string())?;

    let mut fact = Fact::new(scope, fact_type, title, body);
    fact.tags = tags_arg(args);
    fact.description = str_arg(args, "description");
    fact.template = str_arg(args, "template");

    store.write(&fact).map_err(|err| err.to_string())?;
    Ok(format!("Created fact: {}", fact.id))
}

fn supersede(args: &Value) -> Result<String, String> {
    let old_id = require_id(args, "oldId")?;
    let title = require(args, "title")?;
    let body = require(args, "body")?;

    // Inherit type/scope/tags from the predecessor unless explicitly overridden,
    // so a revision can never silently change a fact's scope by omission.
    let predecessor = Store::find_by_id(&old_id).map_err(|err| err.to_string())?;
    let fact_type = type_arg(args)?.unwrap_or_else(|| predecessor.fact_type.clone());
    let scope = match str_arg(args, "scope") {
        Some(_) => scope_arg(args)?,
        None => predecessor.scope.clone(),
    };

    // `supersede` both retires the predecessor and writes the revision, so it
    // needs a store for the revision's scope.
    let store = get_store(&scope).map_err(|err| err.to_string())?;

    let mut fact = Fact::new(scope, fact_type, title, body);
    let tags = tags_arg(args);
    fact.tags = if tags.is_empty() {
        predecessor.tags.clone()
    } else {
        tags
    };
    fact.description = str_arg(args, "description");

    let stored = store
        .supersede(&old_id, fact)
        .map_err(|err| err.to_string())?;
    Ok(format!("Superseded {old_id} with new fact: {}", stored.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mutations_are_refused_server_side_without_allow_writes() {
        for tool in ["uma_write", "uma_supersede"] {
            let err = call(tool, &json!({}), false).unwrap_err();
            assert!(err.contains("--allow-writes"), "{tool}: {err}");
        }
    }

    #[test]
    fn test_unknown_tool_is_reported() {
        assert!(call("uma_nope", &json!({}), true).is_err());
    }

    #[test]
    fn test_missing_required_arguments_are_reported() {
        assert!(call("uma_read", &json!({}), false).is_err());
        assert!(call("uma_search", &json!({}), false).is_err());
        assert!(call("uma_write", &json!({ "title": "x" }), true).is_err());
        assert!(call("uma_supersede", &json!({ "oldId": "nope" }), true).is_err());
    }

    #[test]
    fn test_invalid_ulid_is_rejected_before_touching_the_store() {
        let err = call("uma_read", &json!({ "id": "not-a-ulid" }), false).unwrap_err();
        assert!(err.contains("valid fact ULID"), "{err}");
    }

    #[test]
    fn test_empty_strings_count_as_missing_arguments() {
        let err = call("uma_search", &json!({ "query": "   " }), false).unwrap_err();
        assert!(err.contains("query"), "{err}");
    }
}
