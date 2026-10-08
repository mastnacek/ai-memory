# AGENTS.md — UMA (Universal Memory Architecture)

This document defines the architectural guidelines, development standards, and rules of engagement for all AI coding agents working on the **UMA (ai-memory)** repository.

---

## 1. Project Nature & Scope

- **What UMA Is**: A local-first, high-performance, agent-controlled memory engine designed to give AI coding agents durable, auditable, and structured long-term memory across sessions and projects.
- **Not a Pure Pi Plugin**: UMA is an independent, standalone memory platform written in **Rust** with multi-client delivery:
  - **Standalone CLI (`uma`)**: Direct terminal interface and agent scripting.
  - **Universal MCP Server (stdio JSON-RPC)**: Integrates seamlessly with Claude Code, Cursor, OpenCode, Codex, and any MCP-compliant client.
  - **Pi Extension (`uma-pi-extension`)**: Thin TypeScript wrapper providing native Pi tools (`uma_write`, `uma_recall`, `/uma` panel commands).

---

## 2. Mandatory Architecture: Vertical Slice Architecture (VSA)

This project strictly adopts and enforces **Vertical Slice Architecture (VSA)** based on the standard defined in the `herdr-plugin-dev` skill and plugin (`references/vsa-architecture.md`, `references/rust-plugin.md`).

### Inviolable VSA Rules

1. **Composition Root (`uma-cli/src/main.rs`)**:
   - Acts strictly as the CLI entry point and dispatcher.
   - Parses arguments and delegates execution to the appropriate vertical slice.
   - **Zero business logic, storage access, or data processing** is permitted in `main.rs`.

2. **Shared Kernel / Core (`uma-core`, `uma-cli/src/shared/`)**:
   - Contains domain models (`Fact`, `FactId`, `FactType`, `Scope`, `Validity`), storage engine (`Store`), serialization (YAML frontmatter + Markdown), and cross-cutting helpers (scope resolution, output formatting).
   - **Zero knowledge of individual slices**: Shared code must never import or depend on feature slices.

3. **Feature Slices (`uma-cli/src/slices/<feature>.rs` or `uma-cli/src/slices/<feature>/`)**:
   - Each slice encapsulates **one complete, user-visible capability end-to-end**: CLI arguments definition, business logic, storage mutations, validation, and presentation.
   - **Inviolable Slice Boundary**: **Slices NEVER import each other directly.** All shared contracts and types flow through `uma-core` or `src/shared/`.

4. **File Length Limits**:
   - Source files must remain at or below **400 lines** (soft target 300 lines).
   - Extract cohesive submodules within the slice or shared kernel if approaching limits.

---

## 3. Directory Layout

```text
ai-memory/
├── AGENTS.md                   # This file (Architectural mandate for AI agents)
├── uma-prd.md                  # Product Requirements Document & feature roadmap
├── alr/                        # Memory system research & prior art references
├── docs/                       # Project documentation
└── uma/                        # Rust Workspace
    ├── Cargo.toml              # Workspace manifest
    ├── uma-core/               # Shared Kernel: Core Domain & Storage Engine
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── lib.rs          # Library root & integration tests
    │   │   ├── domain.rs       # Domain entities (Fact, FactId, FactType, Scope, Validity)
    │   │   ├── serialization.rs# YAML Frontmatter + Markdown parser/serializer
    │   │   └── store.rs        # File-system storage engine & path resolvers
    │   └── examples/
    │       └── test_store.rs
    └── uma-cli/                # Command-Line Application (VSA)
        ├── Cargo.toml
        └── src/
            ├── main.rs         # Composition Root & CLI dispatch
            ├── shared/         # Shared CLI utilities (Kernel)
            │   ├── mod.rs
            │   ├── scope.rs    # Scope resolution (project git root vs global)
            │   ├── store_helper.rs # Store factory helpers
            │   └── format.rs   # Fact output & summary printers
            └── slices/         # Vertical Feature Slices (Isolated)
                ├── mod.rs
                ├── write.rs    # Slice S0: Write/Create fact
                ├── read.rs     # Slice S0: Read fact by ID
                ├── list.rs     # Slice S0: List facts by scope/type
                └── search.rs   # Slice S1: BM25 search (SQLite FTS5)
```

---

## 4. Progressive Slice Roadmap (per `uma-prd.md`)

When implementing new capabilities, add them as **new vertical feature slices**:

| Slice | Capability | Implementation Target |
| :--- | :--- | :--- |
| **S0** | Core Store + Write / Read / List | `slices/write.rs`, `slices/read.rs`, `slices/list.rs` (Completed) |
| **S1** | Keyword Search (BM25) | `slices/search.rs` (SQLite FTS5 indexer in `uma-core`) (Completed) |
| **S2** | Semantic Vector Search | `slices/search.rs` (OpenRouter embeddings + Hybrid RRF) (Completed) |
| **S3** | Scopes & Frontmatter Validation | `slices/recall.rs`, validation guards |
| **S4** | Temporal Validity & Supersession | `slices/supersede.rs` |
| **S5** | Consolidation Proposer | `slices/consolidate.rs` (Merge/deduplication proposals) |
| **S6** | Procedural Skill Memory | `slices/skill.rs` (Templates & execution) |
| **S7** | Universal MCP Server | `slices/mcp.rs` (`rmcp` / JSON-RPC stdio server) |
| **S8** | Sync & Transport | `slices/sync.rs` (Git/rsync bundle sync) |

---

## 5. Development & Verification Workflow

- **Run all tests**: `cd uma && cargo test` (must pass 100% with 0 warnings).
- **Build binary**: `cd uma && cargo build --release`.
- **Add a new slice**:
  1. Create `uma-cli/src/slices/<feature>.rs` (or `uma-cli/src/slices/<feature>/mod.rs`).
  2. Define the slice `Args` struct with `clap::Args`.
  3. Implement `pub fn run(args: <Feature>Args) -> Result<()>`.
  4. Register the module in `uma-cli/src/slices/mod.rs`.
  5. Add the subcommand variant in `uma-cli/src/main.rs` dispatch.
  6. Add unit tests for argument parsing and execution.

---

## 6. Agent Memory Guidelines (`uma-memory` Skill)

All agents working on this project should utilize UMA tools (`uma_write`, `uma_read`, `uma_list` or `uma` CLI) according to the `uma-memory` skill:

1. **Capture Decisions & Preferences**:
   - Whenever an architectural choice, design pattern, or user preference is established, record it immediately as a structured fact (`decision`, `preference`, `pattern`).
   - Use `Scope::Project` for codebase-specific rules and `Scope::Global` for cross-cutting user preferences.
2. **Quality Rules**:
   - **Atomic**: One concept per fact.
   - **Descriptive Title**: Searchable, concise summary.
   - **Structured Body**: Markdown with Context, Rule, and Consequences.
   - **Tags**: Lowercase, comma-separated domain keywords (e.g. `vsa, rust, arch`).
3. **Check Memory Before Implementing**:
   - Query existing facts with `uma_list` or `uma list` to respect past architectural decisions before refactoring or introducing new patterns.
