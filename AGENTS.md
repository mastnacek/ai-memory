# AGENTS.md — UMA (Universal Memory Architecture)

This document defines the architectural guidelines, development standards, and rules of engagement for all AI coding agents working on the **UMA (ai-memory)** repository.

---

## 1. Project Nature & Scope

- **What UMA Is**: A local-first, high-performance, agent-controlled memory engine designed to give AI coding agents durable, auditable, and structured long-term memory across sessions and projects.
- **Not a Pure Pi Plugin**: UMA is an independent, standalone memory platform written in **Rust** with multi-client delivery:
  - **Standalone CLI (`uma`)**: Direct terminal interface and agent scripting.
  - **Universal MCP Server (stdio JSON-RPC)**: Integrates seamlessly with Claude Code, Cursor, OpenCode, Codex, and any MCP-compliant client.
  - **Pi Extension (`uma-pi-extension`)**: VSA TypeScript plugin providing native Pi tools (`uma_write`, `uma_read`, `uma_list`, `uma_search`, `uma_supersede`), a fail-closed approval gate, an interactive approval modal, and `/uma` slash commands.

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

3. **Feature Slices (`uma-cli/src/slices/<feature>/`, holding `mod.rs` + `README.md`)**:
   - Each slice encapsulates **one complete, user-visible capability end-to-end**: CLI arguments definition, business logic, storage mutations, validation, and presentation.
   - **Inviolable Slice Boundary**: **Slices NEVER import each other directly.** All shared contracts and types flow through `uma-core` or `src/shared/`.
   - **Every slice folder documents itself**: `slices/<feature>/README.md` states what the slice does, why it exists, and the invariant it must not break (concise — what + why, roughly 15-30 lines). This applies to both the Rust CLI and `uma-pi-extension/src/slices/`. A slice without its `README.md` is incomplete.

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
    │   │   ├── store.rs        # File-system storage engine & path resolvers
    │   │   ├── indexer.rs      # SQLite FTS5 index (rebuildable cache)
    │   │   ├── search.rs       # BM25 / semantic / hybrid RRF
    │   │   ├── embeddings.rs   # OpenRouter embedding client
    │   │   ├── vector_store.rs # fact_embeddings table
    │   │   ├── similarity.rs   # Lexical similarity (tokenize, stem, Jaccard)
│   ├── secrets/        # Credential gate (patterns, placeholder filter, env literals)
│   ├── fastbrain/      # System-1 triage: offline heuristics + Jev transport
    │   │   ├── consolidate.rs  # Duplicate groups + contradiction pairs
    │   │   ├── skill.rs        # Skill template placeholder expansion
    │   │   ├── timeline.rs     # Supersession chain reconstruction
    │   │   └── health.rs       # Read-only index health inspection
    │   ├── tests/
    │   │   └── roundtrip.rs    # Integration roundtrip
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
            │   └── format.rs   # Fact rendering (render_* returns String, print_* wraps it)
            └── slices/         # Vertical Feature Slices (Isolated)
                ├── mod.rs      # Slice registry
                ├── write/      # S0: create a fact
                ├── read/       # S0: read a fact by ID
                ├── list/       # S0: list facts (hides deprecated by default)
                ├── search/     # S1/S2: BM25 + semantic + hybrid RRF
                ├── supersede/  # S4: supersession chain
                ├── migrate/    # S4: OKF v0.2 forward migration
                ├── consolidate/# S5: proposes merges; read-only
                ├── skill/      # S6: template storage + expansion; never executes
                ├── mcp/        # S7: stdio JSON-RPC; read-only unless --allow-writes
                ├── timeline/   # S8: supersession history view
                ├── export/     # S8: OKF bundle / JSON export
                ├── doctor/     # S8: read-only health report
                ├── sync/       # S8: git-backed sync of the global store
                ├── scopes/     # cross-project scope discovery (read-only)
                ├── sessions/   # S9: session browser over pi/claude stores (read-only)
                ├── import/     # S9: session→memory candidates (proposals only, never writes)
                ├── secrets/    # credential gate: scan before anything enters memory (read-only)
                └── recall/     # fastbrain recall gate: whether to search, never a mutation
                # every slice folder = mod.rs (+ helpers) + README.md
```

---

## 4. Progressive Slice Roadmap (per `uma-prd.md`)

When implementing new capabilities, add them as **new vertical feature slices**:

| Slice | Capability | Implementation Target |
| :--- | :--- | :--- |
| **S0** | Core Store + Write / Read / List | `slices/{write,read,list}/` (Completed) |
| **S1** | Keyword Search (BM25) | `slices/search/` (SQLite FTS5 indexer in `uma-core`) (Completed) |
| **S2** | Semantic Vector Search | `slices/search/` (OpenRouter embeddings + Hybrid RRF) (Completed) |
| **S3** | Auto-Recall & Context Injection | `[?]` *On Hold* (Operator preference: on-demand explicit search) |
| **S4** | Temporal Validity & Supersession | `slices/supersede/`, `slices/migrate/` (OKF v0.2 Lifecycle & Chained Supersession) (Completed) |
| **S5** | Consolidation Proposer | `slices/consolidate/` (Read-only merge/deduplication + contradiction proposals) (Completed) |
| **S6** | Procedural Skill Memory | `slices/skill/` (Template storage + expansion; **never executes**) (Completed) |
| **S7** | Universal MCP Server | `slices/mcp/` (hand-rolled JSON-RPC stdio; **read-only unless `--allow-writes`**) (Completed) |
| **S8** | Polish + Sync & Transport | `slices/{timeline,export,doctor,sync}/` (Completed — git-based, global store only) |

---

## 5. Development & Verification Workflow

- **Run all tests**: `cd uma && cargo test` (must pass 100% with 0 warnings).
- **Build binary**: `cd uma && cargo build --release`.
- **Add a new slice**:
  1. Create the slice folder `uma-cli/src/slices/<feature>/`.
  2. Write `mod.rs` with the slice `Args` struct (`clap::Args`) and `pub fn run(args: <Feature>Args) -> Result<()>`.
  3. Write `README.md` — what the slice does, why it exists, and its invariant (mandatory).
  4. Register the module in `uma-cli/src/slices/mod.rs`.
  5. Add the subcommand variant in `uma-cli/src/main.rs` dispatch.
  6. Add unit tests for argument parsing and execution.

---

## 6. Agent Memory Guidelines (two `uma-memory` skills)

UMA ships **two deliberately different skills**. They are split by capability boundary, not
duplicated — do not merge them, and do not copy one into the other:

| Skill | Path | Audience | May contain |
| :--- | :--- | :--- | :--- |
| **General** | `skills/uma-memory/SKILL.md` | any skill-reading agent (CLI / MCP) | only portable material: capture triggers, quality, OKF v0.2, lifecycle, `uma` CLI. **No harness-specific tools, UIs, or commands.** |
| **Pi** | `.pi/skills/uma-memory-pi/SKILL.md` | the Pi agent | Pi-only deltas: `uma_*` tools, approval modal, approval gate, `/uma` commands, `auto-approve`, reload rule. |

The general file must stay correct for an agent that has no Pi tools; naming a Pi-only tool there
would make that agent hunt for something that does not exist. The Pi skill references the general
file by path and adds only what is Pi-specific.

### Approval gate (fail-closed)

A `tool_call` hook (`uma-pi-extension/src/hooks/approval_gate.ts`) guards every memory-mutating
tool (`uma_write`, `uma_supersede`, `uma_consolidate`). A mutation is allowed only when an
interactive approval UI exists (the tool then shows its modal) **or** the operator enabled
auto-approval. In `pi -p`, RPC, JSON, or nested `codemode` calls the write is blocked.
Never bypass the gate by calling the `uma` CLI to force a write — that defeats operator consent.

### Operating rules

1. **Capture Decisions & Preferences**:
   - Whenever an architectural choice, design pattern, or user preference is established, record it as a structured fact (`decision`, `preference`, `pattern`).
   - Use `Scope::Project` for codebase-specific rules and `Scope::Global` for cross-cutting user preferences.
2. **Quality Rules**:
   - **Atomic**: One concept per fact.
   - **Descriptive Title**: Searchable, concise summary.
   - **Structured Body**: Markdown with Context, Rule, and Consequences.
   - **Tags**: Lowercase, comma-separated domain keywords (e.g. `vsa, rust, arch`).
3. **Check Memory Before Implementing**:
   - Query existing facts with `uma_search` / `uma list` to respect past decisions before refactoring or introducing new patterns.
4. **Prefer Supersession Over Duplication**: revise via `uma_supersede`, never by adding a contradicting fact.
5. **Propose, Do Not Impose**: writes should go through the approval modal so the operator can approve, edit, or reject them.
