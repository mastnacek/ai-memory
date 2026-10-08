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
| **S6** | Skill memory (procedural) | Reusable how-to with invocation template | S4 (Completed) |
| **S7** | MCP server (stdio) | Claude Code, Cursor, OpenCode read/write same store | S1, S2 (Completed) |
| **S8** | Polish commands + cross-machine sync | Portable memory | S0 (polish Completed; sync pending Q4) |

**Vertical-slice rule**: Each slice ships a working `uma` CLI command + Pi tool(s) + tests. No "infrastructure slice".

### Delivered beyond the original plan

Capabilities that were not in the roadmap but are implemented, tested and in use:

| Capability | Where | Why it was needed |
| :--- | :--- | :--- |
| Fail-closed approval gate | `uma-pi-extension/src/hooks/approval_gate.ts` | A `tool_call` hook; no interactive UI ⇒ no write. Verified blocking `pi -p`. Guards exactly `{uma_write, uma_supersede}`. |
| Interactive approval modal | `src/shared/modal.ts` + `modal_renderer.ts` | Approve / edit title / body / tags / toggle scope / reject, in `cs` + `en`. Review before consent. |
| Two skills split by audience | `skills/uma-memory/`, `.pi/skills/uma-memory-pi/` | The general skill must stay correct for agents with no Pi tools. |
| `uma list` status filtering | `slices/list/` | Hides deprecated facts by default (mirrors `search`); showing a superseded rule beside its replacement invites acting on stale memory. |
| `uma read --json` | `slices/read/` | Lets the Pi supersede modal prefill the predecessor's real type/scope/tags. |
| Slice READMEs + folder layout | every `slices/<feature>/` | Colocated what/why/invariant docs; see AGENTS.md §2. |
| Integration roundtrip harness | `uma-core/tests/roundtrip.rs` | write → read → list, index → search, deprecation contract. |
| MCP server | `uma-cli/src/slices/mcp/` | Multi-client reach, read-only unless `--allow-writes`. Wire-level tested without spawning a process. |
| Skill memory | `uma-cli/src/slices/skill/`, `uma-core/src/skill.rs` | Procedural memory: `template` frontmatter field + pure placeholder expansion. UMA expands, never executes. |
| `uma timeline` | `slices/timeline/`, `uma-core/src/timeline.rs` | Reconstructs supersession chains (cycle-safe). The history view for `search --as-of`. |
| `uma export` | `slices/export/` | OKF v0.2 bundle or JSON, so memory is never locked into this tool. |
| `uma doctor` | `slices/doctor/`, `uma-core/src/health.rs` | Read-only health report. Found real index drift on its first run. |

### Hardening (defects found by using the tooling on itself)

| Fix | Why it mattered |
| :--- | :--- |
| `uma_supersede` routed through the approval modal | It called the CLI directly and wrote **silently** — a memory mutation with no consent step at all. |
| The supersede modal now carries `template` | A skill's template could be set or replaced without the reviewer ever seeing the value they were approving. |
| `Store::supersede_within` extracted | `Store::supersede` resolves the real global/project roots, so supersession **could not be tested** without writing to live memory. Its integration test now exists. |
| `uma supersede --template`, inherited when omitted | Without it a skill was immutable, and a revision silently dropped the template, leaving the skill unexpandable. |
| `uma list` hides deprecated facts | It printed a superseded rule beside its replacement — the exact hazard supersession exists to remove. |
| `uma_consolidate` removed from the gated set | It only proposes; gating a read would have made approval routine. |
| `doctor` opens the index read-only | `Indexer::open` drops and recreates `facts_fts` on a schema mismatch, so a diagnosis would have silently repaired what it was measuring. |

**Current tally**: 78 Rust tests (39 CLI + 35 core + 4 integration) and 7 TypeScript tests, 0 warnings in both debug and release builds.

### Outstanding work (as of 2026-10-08)

| Item | Slice | Blocked by |
| :--- | :--- | :--- |
| `uma sync push/pull` (the polish commands are done) | S8 | Question 4 (git vs rsync) |
| Import from prior memory systems (§9) | **unlisted** | **Speculative.** Checked 2026-10-08: none of the four source stores exist on this machine (`~/.pi/agent/memory`, `.memsearch/memory`, `~/.engram/vault`, `~/.pi/agent/pi-hermes-memory`), and their formats would have to be reverse-engineered. Revisit only when a real migration is actually needed. |
| Local embedding fallback | S9 | Question 2 |
| Auto-recall / context injection | S3 | On hold by operator preference (deliberate) |
| Pi panel / `/uma status` | S9 | Question 5 (cosmetic, blocks nothing) |
| `supersede` integration test | — | ✅ Done: `Store::supersede_within` |
| Manual Pi test on 2+ models | — | Needs an interactive session |
| Files over the 300-line soft target | — | `search.rs` **394** (nearly the 400 hard limit — split first), `indexer.rs` 338, `store.rs` 327, `consolidate.rs` 311. All under the hard limit, none urgent yet |
| MCP cross-client test | S7 | Needs an external MCP client |

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
- **Pi extension** → native Pi tools (`uma_write`, `uma_read`, `uma_list`, `uma_search`, `uma_supersede`, `uma_consolidate`, `uma_skill_invoke`), an interactive approval modal, and `/uma` slash commands. (Context injection is S3 — on hold by operator preference.)

They share the **same `uma` binary and store**. No duplication.

**MCP tools exposed** — read-only by default; the last two require `--allow-writes`:
```json
{
  "tools": [
    { "name": "uma_read", "description": "Read fact by ID" },
    { "name": "uma_list", "description": "List facts for a scope/type" },
    { "name": "uma_search", "description": "Hybrid search (keyword + semantic)" },
    { "name": "uma_consolidate", "description": "Propose merges/dedups for review" },
    { "name": "uma_write", "description": "Add a fact", "inputSchema": {...} },
    { "name": "uma_supersede", "description": "Replace fact, chain supersession" }
  ]
}
```
A mutation attempted without `--allow-writes` is refused server-side, not merely hidden from `tools/list`.

**Pi extension tools** (mirror MCP + Pi-specific):
- Registered today: `uma_write`, `uma_read`, `uma_list`, `uma_search`, `uma_supersede`, `uma_consolidate`, `uma_skill_invoke`
- `uma_recall` — **not built**: auto-injection is S3, on hold by operator preference
- Slash commands registered today: `/uma search | list | read | reindex | timeline | export | doctor | lang | auto-approve`, each with argument completions
- `/uma status` and a Pi panel remain unimplemented — cosmetic, S9, question 5
- Read-only tools are deliberately ungated: `uma_consolidate` (proposes) and `uma_skill_invoke` (expands). Gating a read would make approval routine and therefore meaningless.

**Consent differs per tool path** — this is a design invariant, not an accident:
- **Pi**: mutations are fail-closed behind an interactive approval modal. No UI ⇒ no write unless `autoApprove`.
- **CLI**: unconditional by design — it is the scriptable interface, so consent is the human typing the command. It must never be used to work around the gate.
- **MCP**: has no modal, so it must not expose write tools without an explicit, launch-time operator opt-in. See §10 question 7.
- **`uma_consolidate` is read-only and therefore ungated** — it proposes, it never applies. Gating a read would make approval routine.

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

### **Week 7: S6 — Skill Memory** ✅
- [x] `FactType::Skill` plus a `template` frontmatter field (OKF extension, like `id`/`scope`/`supersedes`); round-trips through serialization
- [x] `uma skill new --name "docker-build" --template "docker build -t {{tag}} ."` — also reachable as `uma write -t skill --template ...`
- [x] `uma skill list` (shows required placeholders) and `uma skill show <name|id>`
- [x] `uma skill invoke <name|id> --set tag=v1` **expands only** — see question 3 below
- [x] Pi tool `uma_skill_invoke` returns the expanded command as text; the agent runs it through its own shell tool, so the harness's command approval applies
- [x] Expansion is pure and lives in `uma-core/src/skill.rs` (7 unit tests); JSON output carries `"executed": false`
- [x] **Test**: skill stored, recalled by name, template expanded; missing/typo placeholders reported and left visible

### **Week 8: S7 — MCP Server (stdio)** ✅
- [x] `uma mcp serve` → newline-delimited JSON-RPC over stdio; implements `initialize`, `ping`, `tools/list`, `tools/call`
- [x] **Read-only by default**: advertises and allows `uma_read`, `uma_list`, `uma_search`, `uma_consolidate`
- [x] `uma mcp serve --allow-writes` additionally enables `uma_write` and `uma_supersede`
- [x] Hand-rolled protocol instead of `rmcp`: the surface is four methods, so a framework would be more dependency surface than protocol
- [x] Mutations are refused **server-side** without the flag, not merely hidden from `tools/list` — otherwise the flag would be advisory and a client could reach a write by guessing a tool name
- [x] Unit + wire tests: handshake, notification silence, malformed line survival, unknown method, tool-error shape
- [x] Add to a client: `claude mcp add uma -- uma mcp serve`
- [ ] **Test**: write from Claude Code, read in Pi, search in Cursor — requires an external MCP client, so manual

### **Week 9: S8 — Polish & Sync** (polish ✅ · sync pending)

- [x] `uma timeline [--id <ULID>] [--all] [--json]` — reconstructs supersession chains oldest-first, newest chain on top. Read-only, always loads deprecated facts (they *are* the history), cycle-safe
- [x] `uma export [--json | --okf --out <dir>]` — portable JSON, or an OKF v0.2 bundle: one standalone Markdown document per fact at `<dir>/<type>/<id>.md` plus `MANIFEST.json`
- [x] `uma doctor [--json] [--strict]` — read-only health report: roots, file counts, index coverage, schema version, embeddings, orphan embeddings, stale rows. Every non-ok finding names its remedy
- [x] Pi: `/uma timeline`, `/uma export --out <dir>`, `/uma doctor`, with argument completions
- [ ] Git sync: `uma sync push/pull` — **needs question 4 answered**
- [ ] Benchmarks and a migration guide from pi-memory / memorix

**Note**: `doctor` paid for itself immediately. On its first run against this repository it reported *38 indexed rows but 26 files on disk* and *12 stale rows* — leftover pollution from test stores written before the canonicality gate existed. Applying its suggested `uma search "" --reindex` took it to 9/9 ok.

### **Week 10+: S9 — Advanced Features**

> Note: this section previously duplicated S8 verbatim. It now lists the genuinely remaining advanced work.

- [ ] **Import slice** (see §9): `uma import pi-memory | memorix | engram | hermes`, preserving `created_at` and creating supersession chains for conflicts. This was described in prose but **never added to the roadmap table**, so it has no slice number.
- [ ] Local embedding fallback so semantic search survives an OpenRouter outage (question 2)
- [ ] Revisit auto-recall / context injection (S3) once the on-demand path has proven itself
- [ ] `/uma status` and the Pi panel (question 5)

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

1. ~~**OpenRouter embedding model**~~ — **Resolved**: `qwen/qwen3-embedding-8b`.
2. **Fallback**: if OpenRouter unavailable, allow local fallback (e.g., `candle` + `bge-m3` ONNX)? *Open — still the only hard external dependency in the read path.*
3. ~~**Skill invocation**~~ — **Resolved**: expansion only. `uma_skill_invoke` returns command *text*; it never executes. Running it is the agent's call through its own shell tool, so the harness's command approval still applies. UMA does not become a code-execution surface reachable through a memory API.
4. **Sync**: git-based (commits = fact changes) or custom rsync protocol? *Open — blocks S8.*
5. **Pi panel**: TUI (like pi-blackhole) or simple text status? *Open — cosmetic, blocks nothing.*
6. ~~**Model for consolidation proposer**~~ — **Resolved**: none. S5 is deterministic lexical analysis (tokenize → light stem → Jaccard), so it needs no model at all and is fully testable offline.
7. ~~**MCP write policy**~~ — **Resolved**: read-only by default; `--allow-writes` is the explicit launch-time operator opt-in. Without it the server cannot mutate memory, and an attempted mutation is refused in-band rather than silently permitted.

---

## 11. Acceptance Criteria (Definition of Done per Slice)

- [x] `uma <cmd>` works standalone (no Pi)
- [x] Pi tool registered, callable, returns typed result
- [x] Unit tests: domain logic, indexer, search fusion
- [x] Integration test: `uma-core/tests/roundtrip.rs` — write → read → list, index → search, deprecation contract
- [x] Integration test for `supersede`: `Store::supersede_within` is a single-store primitive (retire + revise, never resolving another root), so `uma-core/tests/roundtrip.rs` now covers the full supersession chain with no risk of writing to live memory. `Store::supersede` stays the caller-facing convenience that resolves roots and keeps its cross-scope behaviour (a supersession may move a fact project → global).
- [ ] Manual test in Pi session with 2+ models (Opus, Sonnet, local)
- [x] Docs updated: `slices/<feature>/README.md` — colocated with the slice (what it does, why it exists, its invariant), not a central `docs/` tree

---

## 12. Repository Layout

```
ai-memory/
├── AGENTS.md                     # Architectural mandate for AI agents
├── uma-prd.md                    # This document
├── uma/                          # Rust workspace
│   ├── Cargo.toml                # workspace manifest
│   ├── uma-core/                 # Shared kernel: domain, serialization, store,
│   │   ├── src/                  #   indexer, search, embeddings, vector_store,
│   │   │                         #   similarity, consolidate, skill, timeline, health
│   │   ├── tests/roundtrip.rs    # integration roundtrip
│   └── uma-cli/src/
│       ├── main.rs               # composition root (dispatch only)
│       ├── shared/               # kernel: scope, store_helper, format
│       └── slices/<feature>/     # mod.rs + README.md per slice
├── uma-pi-extension/             # TypeScript: Pi client
│   ├── index.ts                  # composition root (state, hooks, slice wiring)
│   ├── src/shared/               # kernel: client, config, i18n, modal, modal_renderer, state, types
│   ├── src/hooks/approval_gate.ts
│   ├── src/slices/<feature>/     # index.ts + README.md per slice
│   └── test/approval_gate.test.ts
├── skills/uma-memory/            # Harness-agnostic skill
├── .pi/skills/uma-memory-pi/     # Pi-specific skill
└── docs/                         # Project docs (slice docs live in-slice)
```

Note: S7 is implemented as the `uma-cli/src/slices/mcp/` slice rather than a separate `uma-mcp/` crate — AGENTS.md mandates feature slices inside the CLI, and a separate crate would be a horizontal layer. `xtask/` is still not needed.

---

## 13. Next Step

**Your call**: 
- Confirm language choice (Rust core + TS Pi extension)
- Pick embedding model
- Say "go" on S0 — I'll scaffold the workspace and first slice

No other info needed from you unless you want to adjust scope.