//! The MCP (Model Context Protocol) stdio server.
//!
//! Exposes UMA memory to any MCP client — Claude Code, Cursor, OpenCode, Codex —
//! over newline-delimited JSON-RPC on stdio.
//!
//! **Consent model.** MCP has no approval modal, so this server is **read-only by
//! default**. Mutating tools are advertised *and* callable only when the operator
//! starts it with `--allow-writes`; the launch flag is the consent step. A
//! mutation attempted without it is refused server-side, not merely hidden, so a
//! client cannot reach a write by guessing a tool name.
//!
//! This is the same invariant the Pi extension enforces with its approval gate,
//! applied to a different tool path: approval belongs to the path, and each path
//! must obtain consent in the way that fits it.

mod catalog;
mod protocol;
mod tools;

use std::io::{self, BufRead, Write};

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::{json, Value};

use protocol::{
    error, initialize_result, success, tool_result, tools_list_result, Request, INTERNAL_ERROR,
    INVALID_PARAMS, METHOD_NOT_FOUND,
};

#[derive(Args, Debug, Clone)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum McpCommand {
    /// Serve the Model Context Protocol over stdin/stdout
    Serve(ServeArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ServeArgs {
    /// Also expose the mutating tools (uma_write, uma_supersede).
    /// Without this flag the server cannot modify memory at all.
    #[arg(long = "allow-writes")]
    pub allow_writes: bool,
}

/// Executes the MCP vertical slice.
pub fn run(args: McpArgs) -> Result<()> {
    match args.command {
        McpCommand::Serve(serve) => {
            if serve.allow_writes {
                eprintln!("uma mcp: write access ENABLED — this server can modify memory.");
            } else {
                eprintln!("uma mcp: read-only (pass --allow-writes to allow mutations).");
            }
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut out = stdout.lock();
            serve_loop(stdin.lock(), &mut out, serve.allow_writes)
        }
    }
}

/// Runs the request loop against any reader/writer pair, so the wire protocol is
/// testable without spawning a process.
fn serve_loop<R: BufRead, W: Write>(
    input: R,
    out: &mut W,
    allow_writes: bool,
) -> Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request = match Request::parse(&line) {
            Ok(request) => request,
            Err(response) => {
                emit(out, &response)?;
                continue;
            }
        };

        // Notifications are never answered, even on success.
        let is_notification = request.is_notification();
        let response = handle(&request, allow_writes);

        if !is_notification {
            let value = response.unwrap_or_else(|| {
                error(
                    request.id.clone().unwrap_or(Value::Null),
                    INTERNAL_ERROR,
                    "Request produced no response.",
                )
            });
            emit(out, &value)?;
        }
    }
    Ok(())
}

fn emit(out: &mut impl Write, value: &Value) -> Result<()> {
    writeln!(out, "{}", serde_json::to_string(value)?)?;
    out.flush()?;
    Ok(())
}

fn handle(request: &Request, allow_writes: bool) -> Option<Value> {
    let id = request.id.clone().unwrap_or(Value::Null);

    let outcome: Result<Value, (i64, String)> = match request.method.as_str() {
        "initialize" => Ok(initialize_result()),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools_list_result(catalog::specs_as_json(allow_writes))),
        "tools/call" => return Some(call_tool(request, allow_writes)),
        // Client-to-server notifications require no reply.
        method if method.starts_with("notifications/") => return None,
        other => Err((METHOD_NOT_FOUND, format!("Method not found: {other}"))),
    };

    match outcome {
        Ok(result) => Some(success(id, result)),
        Err((code, message)) => Some(error(id, code, &message)),
    }
}

fn call_tool(request: &Request, allow_writes: bool) -> Value {
    let id = request.id.clone().unwrap_or(Value::Null);

    let Some(name) = request.params.get("name").and_then(Value::as_str) else {
        return error(id, INVALID_PARAMS, "tools/call requires a 'name' parameter.");
    };
    let args = request
        .params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // Tool-level failures are reported inside a successful response with
    // `isError: true`, which is what the MCP spec expects.
    match tools::call(name, &args, allow_writes) {
        Ok(text) => success(id, tool_result(text, false)),
        Err(message) => success(id, tool_result(message, true)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, Subcommand};
    use std::io::Cursor;

    #[derive(Parser, Debug)]
    #[command(name = "uma")]
    struct TestCli {
        #[command(subcommand)]
        command: TopCommand,
    }

    #[derive(Subcommand, Debug)]
    enum TopCommand {
        Mcp(McpArgs),
    }

    fn serve_args(argv: &[&str]) -> ServeArgs {
        match TestCli::try_parse_from(argv).expect("should parse").command {
            TopCommand::Mcp(args) => match args.command {
                McpCommand::Serve(serve) => serve,
            },
        }
    }

    fn run_script(script: &str, allow_writes: bool) -> Vec<Value> {
        let mut out = Vec::new();
        serve_loop(Cursor::new(script), &mut out, allow_writes).expect("loop should not fail");
        String::from_utf8(out)
            .expect("utf8")
            .lines()
            .map(|line| serde_json::from_str(line).expect("valid json response"))
            .collect()
    }

    fn tool_names(response: &Value) -> Vec<String> {
        response["result"]["tools"]
            .as_array()
            .expect("tools array")
            .iter()
            .filter_map(|tool| tool["name"].as_str().map(String::from))
            .collect()
    }

    #[test]
    fn test_serve_defaults_to_read_only() {
        assert!(!serve_args(&["uma", "mcp", "serve"]).allow_writes);
        assert!(serve_args(&["uma", "mcp", "serve", "--allow-writes"]).allow_writes);
    }

    #[test]
    fn test_handshake_notifications_and_read_only_tool_list() {
        let script = concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            "\n",
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            "\n",
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            "\n",
        );
        let responses = run_script(script, false);

        // The notification must not produce a response.
        assert_eq!(responses.len(), 2, "got: {responses:?}");
        assert_eq!(responses[0]["result"]["serverInfo"]["name"], "uma");
        assert_eq!(responses[0]["id"], 1);

        let names = tool_names(&responses[1]);
        assert!(names.contains(&"uma_read".to_string()));
        assert!(!names.contains(&"uma_write".to_string()));
    }

    #[test]
    fn test_write_is_advertised_only_with_allow_writes() {
        let script = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let names = tool_names(&run_script(script, true)[0]);
        assert!(names.contains(&"uma_write".to_string()));
        assert!(names.contains(&"uma_supersede".to_string()));
    }

    #[test]
    fn test_guessed_mutating_tool_is_refused_and_reported_as_tool_error() {
        let script = concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"uma_write","arguments":{"title":"x","body":"y"}}}"#,
            "\n",
        );
        let responses = run_script(script, false);
        let result = &responses[0]["result"];

        // A JSON-RPC success carrying an in-band tool error, per the MCP spec.
        assert!(responses[0]["error"].is_null());
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("--allow-writes"));
    }

    #[test]
    fn test_malformed_line_does_not_kill_the_loop() {
        let script = concat!(
            "this is not json\n",
            r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#,
            "\n",
        );
        let responses = run_script(script, false);
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0]["error"]["code"], protocol::PARSE_ERROR);
        assert_eq!(responses[1]["id"], 7);
    }

    #[test]
    fn test_unknown_method_reports_method_not_found() {
        let script = r#"{"jsonrpc":"2.0","id":3,"method":"resources/list"}"#;
        let responses = run_script(script, false);
        assert_eq!(responses[0]["error"]["code"], METHOD_NOT_FOUND);
    }
}
