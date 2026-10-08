# UMA — Unified Memory Architecture
**Product Requirements Document**  
*Vertical-slice, quick-win implementation for Pi agent*

---

## 1. Vision & Scope

**Goal**: A local-first, agent-controlled memory system that grows with the user — start with basic read/write in 1 slice, add search, consolidation, skills, cross-agent sync incrementally.

**Target user**: Pi agent operator (you), using multiple models, wanting durable, auditable memory without cloud lock-in.

**Non-goals**: Cloud service, multi-tenant SaaS, LLM-as-a-service.

---

## 2. Architecture: Vertical Slices

Each slice = **one user-visible capability** + its data, tools, storage, tests. No horizontal layers.

| Slice | Capability | User Value | Depends On |
| :--- | :--- | :--- | :--- |
| **S0** | Core store + `uma_write` / `uma_read` | Persist one fact, read it back | — (Completed) |
| **S1** | `uma_search` (keyword BM25) | Find facts by term | S0 (Completed) |
| **S2** | `uma_search` (semantic vectors) | Find by meaning | S1 (Completed) |
| **S3** | Auto-Recall & Context Injection | Auto-inject relevant facts into turn | S2 (On Hold / `[?]`) |
| **S4** | Temporal validity + Supersession | "What was true on 2026-01-15?", replace facts | S0, S1, S2 (Completed) |
| **S5** | Consolidation proposer (agent-reviewed) | Merge dupes, fix contradictions | S4 (Completed) |
| **S6** | Skill memory (procedural) | Reusable how-to with invocation template | S4 |
| **S7** | MCP server (stdio) | Claude Code, Cursor, OpenCode read/write same store | S1, S2 |
| **S8** | Cross-machine sync (git/rsync) | Portable memory | S0 |

**Vertical-slice rule**: Each slice ships a working `uma` CLI command + Pi tool(s) + tests. No "infrastructure slice".

---

## 3. Data Model

```fsharp
// File: uma-core/src/Domain.fs
type FactId = FactId of string           // ULID
type Scope = Project of string | Global
type FactType = Decision | Preference | Fact | Skill | Correction
type Validity = { ValidFrom: DateTime; InvalidAt: DateTime option }

type Fact = {
    Id: FactId
    Scope: Scope
    Type: FactType
    Title: string           // one-line, human-readable
    Body: string            // Markdown
    Tags: string list
    Validity: Validity
    Supersedes: FactId option
    EvidenceRefs: string list   // session ids, file paths, URLs
    CreatedAt: DateTime
    ModifiedAt: DateTime
}

// On disk: one file per fact at <root>/<scope>/<type>/<id>.md
// Frontmatter = YAML of all fields except Body
// Indexes: SQLite FTS5 (keyword) + LanceDB (vectors) — rebuildable
```

---

## 4. Language & Runtime Decision

| Preference | Verdict |
|------------|---------|
| **Rust** | ✅ **Primary** — best for CLI performance, embedding (LanceDB via `lancedb-rs`), SQLite (`rusqlite`), MCP server (`rmcp`), Pi extension (NAPI via `napi-rs` or separate Node wrapper). |
| **F#** | ❌ Not viable for Pi extension (no NAPI story), MCP server harder, LanceDB no native binding. |
| **TypeScript** | ✅ **Pi extension entry point** — Pi loads TS/JS extensions natively. Thin wrapper calling Rust core via NAPI or stdio. |
| **Python** | ❌ Unnecessary; Rust covers all. |

**Implementation plan**:
- **Core + CLI + MCP server** → **Rust** (single binary `uma`)
- **Pi extension** → **TypeScript** (`uma-pi-extension`) calling `uma` binary via `child_process` or NAPI module
- **Embeddings** → **OpenRouter API** — model selection via OpenRouter (e.g., `openai/text-embedding-3-large`, `nomic-embed-text`, `jina-embeddings-v3`), HTTP calls from Rust, no local model dependencies

---

## 5. Delivery Format: MCP + Pi Extension (Both)

**Why both?**
- **MCP (stdio)** → universal: Claude Code, Cursor, OpenCode, Codex, any MCP client
- **Pi extension** → native Pi tools (`uma_write`, `uma_search`, `uma_recall`), context injection, `/uma` panel commands

They share the **same `uma` binary and store**. No duplication.

**MCP tools exposed**:
```json
{
  "tools": [
    { "name": "uma_write", "description": "Add/replace a fact", "inputSchema": {...} },
    { "name": "uma_read", "description": "Read fact by ID" },
    { "name": "uma_search", "description": "Hybrid search (keyword + semantic)" },
    { "name": "uma_supersede", "description": "Replace fact, chain supersession" },
    { "name": "uma_consolidate", "description": "Propose merges/dedups for review" }
  ]
}
```

**Pi extension tools** (mirror MCP + Pi-specific):
- `uma_write`, `uma_read`, `uma_search`, `uma_supersede`, `uma_consolidate`
- `uma_recall` — injects top-N relevant facts into next turn (like pi-memory auto-surfacing)
- Slash commands: `/uma status`, `/uma timeline`, `/uma export`, `/uma doctor`

---

## 6. Quick-Win Implementation Plan

### **Week 1: S0 — Core Store + Write/Read**
- [x] Rust crate `uma-core`: `Fact` struct, serialization, file layout
- [x] Binary `uma` with `uma write --type decision --title "..." --body "..."` and `uma read <id>`
- [x] Project detection (git root) → scope `Project::<repo>`
- [x] Global scope at `~/.uma/global/`
- [x] Pi extension scaffold: registers `uma_write` / `uma_read` / `uma_list` tools calling `uma` binary
- [x] **Test**: `uma write -t fact -T "test" -b "hello" && uma read <id>` → works in Pi

### **Week 2: S1 — Keyword Search (BM25)**
- [x] SQLite FTS5 index (`uma-core/src/indexer.rs`) — auto-updated on write
- [x] `uma search "query" [--scope project|global] [--type type]`
- [x] Pi tool `uma_search` with BM25 keyword matching
- [x] **Test**: write facts, search returns BM25-ranked results with snippets

### **Week 3: S2 — Semantic Search (Vectors via OpenRouter)**
- [x] Embedding pipeline: HTTP client → OpenRouter `/v1/embeddings` endpoint (`qwen/qwen3-embedding-8b` / `openai/text-embedding-3-large`)
- [x] Vector storage table in centralized SQLite database (`fact_embeddings` in `index.db`)
- [x] `uma search "..." --mode semantic` and `--mode hybrid` (Reciprocal Rank Fusion RRF)
- [x] Auto-discovery of API key from `OPENROUTER_API_KEY`, `~/.pi/agent/auth.json`, or `openrouter-accounts.json`
- [x] Batch vectorization (`uma search --vectorize`)
- [x] **Test**: verified semantic search & hybrid RRF scoring on real facts via OpenRouter API

### **Week 4: S3 — Auto-Recall & Context Injection [?]**
- [?] *ON HOLD (Operator preference)*: Auto-injecting facts into turns is paused to prevent context noise and hallucinations. On-demand search via `uma_search` is prioritized.

### **Week 5: S4 — Temporal Validity, OKF Lifecycle & Supersession** ✅
- [x] Align YAML frontmatter with OKF v0.2 specification (`type`, `title`, `description`, `tags`, `status`, `generated`, `verified`)
- [x] `since` (valid_from), `until` (invalid_at), and `stale_after` in frontmatter
- [x] `uma supersede <old-id> --title "..." --body "..."` → sets old fact `status: deprecated`, sets `until`, and chains `supersedes: <old_id>`
- [x] `uma search --as-of "2026-01-01T00:00:00Z"` filters by validity and hides deprecated facts from default search
- [x] `uma search --include-deprecated` surfaces superseded history with a `[DEPRECATED]` badge
- [x] `uma migrate [--dry-run] [--reindex]` rewrites legacy markdown files into the OKF v0.2 layout and backfills `generated`/`verified`
- [x] Pi tool `uma_supersede` + `uma_search` gained `includeDeprecated` and `asOf`
- [x] FTS5 schema versioning: an old `index.db` is auto-rebuilt (the index is a rebuildable cache)
- [x] **Test**: supersede chain queryable, deprecated version hidden by default, precise `--as-of` time-travel verified end-to-end

### **Week 6: S5 — Consolidation Proposer** ✅
- [x] `uma consolidate [--scope project] [--type t] [--threshold 0.55] [--json]` → prints proposed merges (overlapping title/body wording) and contradiction flags (similar wording, opposite polarity)
- [x] Deterministic offline lexical analysis (`uma-core/src/similarity.rs` + `consolidate.rs`) — embeddings are an enhancement, never a prerequisite, so this slice is fully unit-testable
- [x] Agent reviews via the `uma_consolidate` tool → returns proposals; the agent accepts with `uma_write` / `uma_supersede`, which are the gated tools
- [x] Read-only by construction: the slice never writes, so it is deliberately **not** behind the approval gate
- [x] **Test**: two "use pnpm" facts → proposer merges; "Use pnpm" vs "Do not use pnpm" → contradiction flag; unrelated facts → nothing

### **Week 7: S6 — Skill Memory**
- [ ] `FactType::Skill` with extra field `InvocationTemplate: string`
- [ ] `uma skill new --name "docker-build" --template "docker build -t {{tag}} ."`
- [ ] Pi tool `uma_skill_invoke` expands template + runs
- [ ] **Test**: skill recalled, template filled, executed

### **Week 8: S7 — MCP Server (stdio)**
- [ ] `uma mcp serve` → stdio JSON-RPC, registers 5 tools
- [ ] Test with Claude Code: `claude mcp add uma -- uma mcp serve`
- [ ] **Test**: write from Claude Code, read in Pi, search in Cursor

### **Week 9: S8 — Polish & Sync**
- [ ] `/uma timeline`, `/uma export --okf`, `/uma doctor`
- [ ] Git sync: `uma sync push/pull` (bundles facts as commits)
- [ ] Benchmarks, docs, migration guide from pi-memory/memorix

### **Week 10+: S9 — Advanced Features**
- [ ] `/uma timeline`, `/uma export --okf`, `/uma doctor`
- [ ] Git sync: `uma sync push/pull` (bundles facts as commits)
- [ ] Benchmarks, docs, migration guide from pi-memory/memorix

---

## 7. Configuration

```toml
# ~/.uma/config.toml  (global)  +  .uma/config.toml (per-project, overrides)
[store]
root = "~/.uma"                    # global root
project_root = ".uma"              # per-project (gitignored)

[index]
keyword = true                     # SQLite FTS5
semantic = true                    # LanceDB vectors
embedding_provider = "openrouter"  # openrouter | local
embedding_model = "openai/text-embedding-3-large"  # OpenRouter model ID
embedding_batch = 32
embedding_dimensions = 3072        # depends on model (3072 for text-embedding-3-large, 768 for nomic, 1024 for jina-v3)
embedding_api_key_env = "OPENROUTER_API_KEY"  # env var for API key

[recall]
project_limit = 5
global_limit = 3
mode = "hybrid"                    # keyword | semantic | hybrid
max_tokens = 4000                  # budget for injection

[consolidation]
auto_propose = true                # after each turn
similarity_threshold = 0.85        # for dedup detection
contradiction_check = true

[mcp]
enabled = true
transport = "stdio"
```

---

## 7b. OpenRouter Embedding Integration

**Endpoint**: `https://openrouter.ai/api/v1/embeddings` (OpenAI-compatible)

**Request**:
```json
{
  "model": "openai/text-embedding-3-large",
  "input": ["text to embed", "another text"],
  "encoding_format": "float"
}
```

**Response**:
```json
{
  "object": "list",
  "data": [
    {"object": "embedding", "index": 0, "embedding": [0.1, -0.3, ...]}
  ],
  "model": "openai/text-embedding-3-large",
  "usage": {"prompt_tokens": 10, "total_tokens": 10}
}
```

**Rust Implementation** (`uma-core/src/embeddings.rs`):
- `reqwest` HTTP client with connection pooling
- Retry with exponential backoff (max 3 retries)
- Batch up to `embedding_batch` texts per request
- Rate limit handling: respect `Retry-After` header, exponential backoff on 429
- Timeout: 30s default, configurable
- Cache: in-memory LRU cache keyed by text hash (avoid re-embedding identical content)

**Error Handling**:
- 401 → invalid API key → clear error to user
- 402 → credits exhausted → clear error
- 429 → rate limited → retry with backoff
- 5xx → transient → retry
- Network error → retry

**Cost Estimation** (OpenRouter pricing, Oct 2026):
| Model | Cost / 1M tokens | Dim | Notes |
|-------|------------------|-----|-------|
| `openai/text-embedding-3-large` | ~$0.13 | 3072 | Best quality |
| `openai/text-embedding-3-small` | ~$0.02 | 1536 | Good quality, cheaper |
| `nomic-embed-text` | ~$0.05 | 768 | Open weights, good |
| `jina-embeddings-v3` | ~$0.08 | 1024 | Multilingual, good |
| `mistral-embed` | ~$0.10 | 1024 | Strong retrieval |

**Decision**: Default to `text-embedding-3-large` for quality; user can override via config.

---

## 8. Pi Extension Structure

```
uma-pi-extension/
├── package.json
├── index.ts                  # Composition root: state, hooks, slice wiring
├── src/
│   ├── shared/               # Kernel
│   │   ├── client.ts         # spawns `uma`, parses JSON
│   │   ├── config.ts         # .pi/uma.json + global config
│   │   ├── i18n.ts           # cs/en strings
│   │   ├── modal.ts          # approval modal
│   │   ├── modal_renderer.ts # modal drawing
│   │   ├── state.ts
│   │   └── types.ts
│   ├── hooks/
│   │   └── approval_gate.ts  # fail-closed tool_call guard
│   └── slices/
│       ├── write/     { index.ts, README.md }
│       ├── read/      { index.ts, README.md }
│       ├── list/      { index.ts, README.md }
│       ├── search/    { index.ts, README.md }
│       ├── supersede/ { index.ts, README.md }
│       └── commands/  { index.ts, complete.ts, README.md }
└── test/
    └── approval_gate.test.ts
```

**Key point**: Pi extension is *thin* — all logic in Rust binary. Extension only handles Pi protocol, tool schemas, consent (the approval modal), and slice wiring.

---

## 9. Migration Path (for you)

| From | Command |
|------|---------|
| pi-memory | `uma import pi-memory --source ~/.pi/agent/memory` |
| memorix | `uma import memorix --source .memsearch/memory` |
| gentle-engram | `uma import engram --source ~/.engram/vault` |
| pi-hermes-memory | `uma import hermes --source ~/.pi/agent/pi-hermes-memory` |

Imports preserve `created_at`, map types, create supersession chains for conflicts.

---

## 10. Open Questions for You

1. **OpenRouter embedding model**: `openai/text-embedding-3-large` (3072-dim) OK, or prefer `nomic-embed-text` (768-dim) / `jina-embeddings-v3` (1024-dim) / `mistral-embed` (1024-dim)?
2. **Fallback**: if OpenRouter unavailable, allow local fallback (e.g., `candle` + `bge-m3` ONNX)?
3. **Skill invocation**: template expansion only, or also allow shell command templates?
4. **Sync**: git-based (commits = fact changes) or custom rsync protocol?
5. **Pi panel**: TUI (like pi-blackhole) or simple text status?
6. **Model for consolidation proposer**: reuse Pi's active model, or dedicated cheap model (e.g., `qwen3-8b` via Ollama)?

---

## 11. Acceptance Criteria (Definition of Done per Slice)

- [ ] `uma <cmd>` works standalone (no Pi)
- [ ] Pi tool registered, callable, returns typed result
- [ ] Unit tests: domain logic, indexer, search fusion
- [ ] Integration test: write → search → read → supersede roundtrip
- [ ] Manual test in Pi session with 2+ models (Opus, Sonnet, local)
- [x] Docs updated: `slices/<feature>/README.md` — colocated with the slice (what it does, why it exists, its invariant), not a central `docs/` tree

---

## 12. Repository Layout

```
uma/
├── Cargo.toml                    # workspace
├── uma-core/                     # Rust: domain, store, indexer, search, consolidation
├── uma-cli/                      # Rust: CLI entry (uma)
├── uma-mcp/                      # Rust: MCP stdio server (rmcp)
├── uma-pi-extension/             # TypeScript: Pi extension
├── docs/
│   ├── architecture.md
│   ├── slice-0-core.md
│   ├── slice-1-keyword-search.md
│   └── ...
├── tests/
│   ├── integration/              # end-to-end via CLI + Pi
│   └── fixtures/
└── xtask/                        # Rust build automation
```

---

## 13. Next Step

**Your call**: 
- Confirm language choice (Rust core + TS Pi extension)
- Pick embedding model
- Say "go" on S0 — I'll scaffold the workspace and first slice

No other info needed from you unless you want to adjust scope.