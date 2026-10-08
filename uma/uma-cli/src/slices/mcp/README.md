# `mcp` slice

**Command:** `uma mcp serve [--allow-writes]`
**Depends on:** `uma-core::{store, search, consolidate, domain}`

## What it does
Serves UMA memory to any MCP client (Claude Code, Cursor, OpenCode, Codex) over newline-delimited JSON-RPC on stdin/stdout. Implements four methods: `initialize`, `ping`, `tools/list`, `tools/call`.

| Tool | Mode | Purpose |
| :--- | :--- | :--- |
| `uma_read` | read-only | Read one fact by ULID |
| `uma_list` | read-only | Enumerate facts for a scope/type |
| `uma_search` | read-only | keyword / semantic / hybrid search |
| `uma_consolidate` | read-only | Propose merges and flag contradictions |
| `uma_write` | **`--allow-writes`** | Create a fact |
| `uma_supersede` | **`--allow-writes`** | Replace a fact, chaining supersession |

The protocol is hand-rolled rather than framework-based: the surface is four methods over line-delimited JSON, so a server framework would be more dependency surface than protocol.

## Why it exists
This is the multi-client reach that makes UMA a platform instead of a Pi feature. One store, many harnesses — no per-harness copy of memory.

## Invariant
- **Read-only by default.** MCP has no approval modal, so the launch flag is the consent step. `--allow-writes` is required to advertise *and* to call any mutating tool.
- A mutation attempted without the flag is **refused server-side**, not merely omitted from `tools/list` — otherwise the flag would be advisory and a client could reach a write by guessing the tool name.
- Notifications are never answered, tool failures return `isError: true` inside a successful JSON-RPC response, and a malformed line yields a parse error without killing the loop.
