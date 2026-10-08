//! Minimal JSON-RPC 2.0 and MCP message shapes for the stdio transport.
//!
//! Hand-rolled rather than pulling in a server framework: the MCP surface this
//! server needs is four methods over newline-delimited JSON, and a dependency
//! tree would be far more surface area than the protocol itself.
//!
//! MCP stdio framing: one JSON message per line, no `Content-Length` headers.

use serde::Deserialize;
use serde_json::{json, Value};

pub const JSONRPC_VERSION: &str = "2.0";

/// Protocol revision this server implements.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

pub const PARSE_ERROR: i64 = -32700;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

/// An inbound JSON-RPC message. Notifications carry no `id`.
#[derive(Debug, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

impl Request {
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    /// Parses one transport line, returning a ready-made error response on failure.
    pub fn parse(line: &str) -> Result<Self, Value> {
        serde_json::from_str(line)
            .map_err(|err| error(Value::Null, PARSE_ERROR, &format!("Parse error: {err}")))
    }
}

/// Builds a successful JSON-RPC response.
pub fn success(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": JSONRPC_VERSION, "id": id, "result": result })
}

/// Builds a JSON-RPC error response.
pub fn error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": { "code": code, "message": message }
    })
}

/// The MCP `initialize` result.
pub fn initialize_result() -> Value {
    json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "uma",
            "version": env!("CARGO_PKG_VERSION"),
        }
    })
}

/// The MCP `tools/list` result.
pub fn tools_list_result(tools: Vec<Value>) -> Value {
    json!({ "tools": tools })
}

/// The MCP `tools/call` result, carrying a single text block.
pub fn tool_result(text: String, is_error: bool) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_parsing_detects_notifications() {
        let call = Request::parse(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#).unwrap();
        assert_eq!(call.method, "tools/list");
        assert!(!call.is_notification());

        let note =
            Request::parse(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).unwrap();
        assert!(note.is_notification());
    }

    #[test]
    fn test_malformed_input_yields_a_parse_error_response() {
        let err = Request::parse("not json").unwrap_err();
        assert_eq!(err["error"]["code"], PARSE_ERROR);
        assert!(err["id"].is_null());
    }

    #[test]
    fn test_initialize_result_advertises_tools_capability() {
        let result = initialize_result();
        assert_eq!(result["protocolVersion"], MCP_PROTOCOL_VERSION);
        assert!(result["capabilities"]["tools"].is_object());
        assert_eq!(result["serverInfo"]["name"], "uma");
    }

    #[test]
    fn test_tool_result_shape() {
        let result = tool_result("hello".to_string(), false);
        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(result["content"][0]["text"], "hello");
        assert_eq!(result["isError"], false);
    }
}
