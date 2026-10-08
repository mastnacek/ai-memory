//! The MCP tool catalogue.
//!
//! Tools are split by whether they mutate the store:
//!
//! - **Read-only** — always advertised: `uma_read`, `uma_list`, `uma_search`,
//!   `uma_consolidate`.
//! - **Mutating** — advertised *only* with `--allow-writes`: `uma_write`,
//!   `uma_supersede`.
//!
//! MCP has no approval modal, so the launch flag is the consent step: the
//! operator decides once, at start-up, whether this server may rewrite memory.

use serde_json::{json, Value};

/// A tool the server can advertise.
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub mutating: bool,
    pub schema: Value,
}

/// Returned when a mutation is attempted without `--allow-writes`.
pub const WRITE_REFUSAL: &str =
    "Refused: this tool mutates UMA memory and the server was started without write access. \
     Restart it with `uma mcp serve --allow-writes` if writes are intended.";

/// True when the tool mutates the store.
///
/// Consulted by the dispatcher before every `tools/call`, so this is the
/// enforcement path rather than documentation. It reads the same `mutating`
/// flag that `tools/list` advertises, which keeps the advertised catalogue and
/// the server-side rule from ever drifting apart.
pub fn is_mutating(name: &str) -> bool {
    specs(true)
        .into_iter()
        .any(|tool| tool.name == name && tool.mutating)
}

fn read_only_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "uma_read",
            description: "Read one UMA memory fact by its ULID identifier.",
            mutating: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Fact ULID." }
                },
                "required": ["id"]
            }),
        },
        ToolSpec {
            name: "uma_list",
            description:
                "List UMA memory facts for a scope, optionally filtered by type. Deprecated facts are hidden unless includeDeprecated is set.",
            mutating: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "scope": { "type": "string", "description": "'global' or a project name. Defaults to the server's project." },
                    "type": { "type": "string", "description": "Fact type: decision, preference, pattern, skill, note, ..." },
                    "includeDeprecated": { "type": "boolean", "description": "Include deprecated/superseded facts." }
                }
            }),
        },
        ToolSpec {
            name: "uma_search",
            description:
                "Search UMA memory by keyword (BM25), meaning (semantic vectors), or hybrid RRF fusion.",
            mutating: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Keywords or a semantic concept." },
                    "mode": { "type": "string", "enum": ["keyword", "semantic", "hybrid"], "description": "Default: hybrid." },
                    "scope": { "type": "string" },
                    "type": { "type": "string" },
                    "limit": { "type": "integer", "description": "Max results (default 10)." },
                    "asOf": { "type": "string", "description": "ISO 8601 point-in-time filter." },
                    "includeDeprecated": { "type": "boolean" }
                },
                "required": ["query"]
            }),
        },
        ToolSpec {
            name: "uma_consolidate",
            description:
                "Review UMA memory for near-duplicate and contradicting facts. Read-only: it proposes, it never applies. Apply proposals with uma_supersede or uma_write.",
            mutating: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "scope": { "type": "string" },
                    "type": { "type": "string" },
                    "threshold": { "type": "number", "description": "Similarity threshold 0.0-1.0 (default 0.55)." },
                    "includeDeprecated": { "type": "boolean" }
                }
            }),
        },
    ]
}

fn mutating_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "uma_write",
            description: "Create a new UMA memory fact.",
            mutating: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "body": { "type": "string", "description": "Markdown body." },
                    "type": { "type": "string", "description": "Default: note." },
                    "scope": { "type": "string" },
                    "tags": { "type": "array", "items": { "type": "string" } },
                    "template": { "type": "string", "description": "For type 'skill': an invocation template with {{placeholders}}." }
                },
                "required": ["title", "body"]
            }),
        },
        ToolSpec {
            name: "uma_supersede",
            description:
                "Replace an existing UMA fact with a revision. The predecessor is deprecated, never deleted, and the new fact links back to it.",
            mutating: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "oldId": { "type": "string", "description": "ULID of the fact being replaced." },
                    "title": { "type": "string" },
                    "body": { "type": "string" },
                    "type": { "type": "string", "description": "Defaults to the predecessor's type." },
                    "scope": { "type": "string", "description": "Defaults to the predecessor's scope." },
                    "tags": { "type": "array", "items": { "type": "string" } },
                    "description": { "type": "string" }
                },
                "required": ["oldId", "title", "body"]
            }),
        },
    ]
}

/// The tools to advertise. Mutating tools are omitted unless writes are allowed.
pub fn specs(allow_writes: bool) -> Vec<ToolSpec> {
    let mut tools = read_only_specs();
    if allow_writes {
        tools.extend(mutating_specs());
    }
    tools
}

/// Serialises the catalogue into the shape MCP `tools/list` expects.
pub fn specs_as_json(allow_writes: bool) -> Vec<Value> {
    specs(allow_writes)
        .into_iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.schema,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(allow_writes: bool) -> Vec<&'static str> {
        specs(allow_writes).into_iter().map(|t| t.name).collect()
    }

    #[test]
    fn test_read_only_catalogue_omits_mutating_tools() {
        let names = names(false);
        assert!(names.contains(&"uma_read"));
        assert!(names.contains(&"uma_list"));
        assert!(names.contains(&"uma_search"));
        assert!(names.contains(&"uma_consolidate"));
        assert!(!names.contains(&"uma_write"));
        assert!(!names.contains(&"uma_supersede"));
    }

    #[test]
    fn test_allow_writes_advertises_mutating_tools() {
        let names = names(true);
        assert!(names.contains(&"uma_write"));
        assert!(names.contains(&"uma_supersede"));
        // Read-only tools must never disappear when writes are enabled.
        assert!(names.contains(&"uma_read"));
    }

    #[test]
    fn test_exactly_the_expected_tools_are_mutating() {
        // Pins the real contract by name: getting this wrong either misleads a
        // client or silently permits a write without the operator's opt-in.
        for name in ["uma_read", "uma_list", "uma_search", "uma_consolidate"] {
            assert!(!is_mutating(name), "{name} must not be mutating");
        }
        for name in ["uma_write", "uma_supersede"] {
            assert!(is_mutating(name), "{name} must be mutating");
        }
        assert!(!is_mutating("uma_unknown"));
    }

    #[test]
    fn test_specs_as_json_matches_mcp_tool_shape() {
        let list = specs_as_json(false);
        assert!(!list.is_empty());
        for tool in &list {
            assert!(tool["name"].is_string());
            assert!(tool["description"].is_string());
            assert!(tool["inputSchema"].is_object());
        }
    }
}
