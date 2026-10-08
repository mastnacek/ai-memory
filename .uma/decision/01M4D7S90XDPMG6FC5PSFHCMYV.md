---
id: 01M4D7S90XDPMG6FC5PSFHCMYV
scope: "project:ai-memory"
type: decision
title: Core Engine in Rust with Multi-Client Delivery
since: "2026-10-08T07:48:58.525562400+00:00"
tags:
  - tech-stack
  - rust
  - cli
  - mcp
  - pi-extension
---
### Context
We need a high-performance, local-first memory system that works seamlessly across multiple AI coding agents and harnesses (Pi, Claude Code, Cursor, OpenCode, Codex, and terminal CLI).

### Decision
- **Core Engine & CLI (`uma`)**: Written in Rust for sub-10ms startup, memory efficiency, embedded SQLite/LanceDB, and single-binary distribution.
- **Universal MCP Server**: Integrated into the Rust binary (`uma mcp serve` via `rmcp`) for Claude Code, Cursor, etc.
- **Pi Integration**: Thin TypeScript extension (`.pi/extensions/uma.ts`) that invokes the `uma` binary.
- **Embeddings**: OpenRouter API (`openai/text-embedding-3-large`, `nomic-embed-text`, etc.) for vector embeddings without heavy local model dependencies.

### Consequences
- Single source of truth for all tools and CLI commands in Rust.
- Zero duplicate logic between Pi extension and MCP server.