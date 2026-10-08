---
name: uma-memory
description: Store, search, and recall structured project decisions, user preferences, architectural patterns, and reusable skills using the UMA memory engine.
---

# UMA Memory Skill (harness-agnostic)

This skill guides **any** coding agent on when, why, and how to capture, organize, and retrieve
structured long-term memories with the **UMA (Universal Memory Architecture)** engine.

> **Scope: portable UMA only.** This file deliberately describes no particular harness. It is
> safe for any agent that can read skills and drive the `uma` CLI (or an MCP client).
> **Do not add harness-specific tool names, UIs, or slash commands here** — an agent that does
> not have them would look for tools that do not exist.

---

## 1. When to Capture Memory

Record a memory under the following triggers:

| Event Trigger | Fact Type | Scope | Example |
| :--- | :--- | :--- | :--- |
| **Architectural Decision** | `decision` | `project` | Choosing VSA, database selection, API protocols. |
| **User Preference** | `preference` | `global` | "Always use Rust & Ratatui for TUI tools", "Prefer concise explanations". |
| **Codebase Pattern** | `pattern` | `project` | Error handling conventions, module isolation rules, test layout. |
| **Correction / Mistake Fix** | `correction` | `project` / `global` | Correcting a false assumption, fixing a recurring bug pattern. |
| **Environmental Fact** | `fact` | `project` | API endpoint URLs, build requirements, port allocations. |
| **Procedural How-To** | `skill` | `project` / `global` | Step-by-step commands to build, package, or deploy the artifact. |

### What NOT to Store

- Ephemeral chat chatter or greetings.
- Raw temporary code dumps that will change in the next turn.
- Obvious information already fully documented in standard files (e.g., standard `Cargo.toml` dependencies).

### Prefer Supersession Over Duplication

When a fact changes, do **not** write a second, contradicting fact. Supersede it (section 4) so
the predecessor is retired with `status: deprecated` and the new revision chains back to it via
`supersedes`. History stays auditable and default search stays clean.

---

## 2. Memory Quality Standards

1. **Atomic & Focused**: One decision or fact per memory entry.
2. **Clear Actionable Title**:
   - Good: `Adopt Vertical Slice Architecture for CLI modules`
   - Bad: `Architecture notes`
3. **Structured Markdown Body**:
   - **Context**: why this was decided or what problem it solves.
   - **Decision / Rule**: the exact convention or constraint to follow.
   - **Consequences**: what to do and what to avoid.
4. **One-line `description`**: a short summary used for previews and search snippets.
5. **Relevant Tags**: lowercase ASCII keywords (`tags: ["vsa", "architecture", "rust"]`).

---

## 3. Frontmatter: OKF v0.2 + UMA Extensions

UMA facts are valid **Open Knowledge Format v0.2** documents (Markdown + YAML frontmatter),
extended with UMA-specific keys.

| Key | Source | Meaning |
| :--- | :--- | :--- |
| `type` | OKF | Concept type (`decision`, `preference`, `pattern`, ...) |
| `title` | OKF | Human-readable name |
| `description` | OKF | One-line summary used for previews/snippets |
| `tags` | OKF | Cross-cutting categorization |
| `status` | OKF | `stable` (default) \| `deprecated` \| `draft` |
| `generated` | OKF | `{ by: <actor>, at: <iso8601> }` — who produced it |
| `verified` | OKF | `[{ by: <actor>, at: <iso8601> }]` — derives the trust tier |
| `since` / `until` / `stale_after` | OKF lifecycle | Validity window and staleness instant |
| `id` | UMA | ULID identifier |
| `scope` | UMA | `global` or `project:<name>` |
| `supersedes` | UMA | ULID of the replaced predecessor fact |

**Actors** follow OKF: `<producer>/<version>` for agents (`my-agent/1.0`),
`human:<id>` for people, `process:<id>` for automation.

**Trust tier** derives from `verified`: absent → unverified; only non-`human:` actors →
machine-confirmed; a `human:<id>` actor → **human-reviewed**.

**On disk**: project facts live at `<repo>/.uma/<type>/<id>.md`; global facts live in the user
profile. The searchable index (FTS5 + vectors) is a single centralized, *rebuildable* cache in
the user profile — never a file inside the repository.

---

## 4. Lifecycle and Retrieval

**Superseding** a fact retires the predecessor (`status: deprecated` + `until`) and chains the
new revision via `supersedes`. Superseded facts are hidden from default search.

**Retrieval modes**:

| Mode | Best for |
| :--- | :--- |
| `keyword` | Exact terms, identifiers, flag names. |
| `semantic` | Meaning-based lookups that share no keywords with the stored title. |
| `hybrid` (default) | Reciprocal Rank Fusion of both — the safest default. |

Prefer **search on demand** over automatic injection: auto-injecting memories adds context noise
and risks acting on stale or irrelevant facts.

---

## 5. CLI Reference

```bash
# Hybrid search (BM25 + Semantic OpenRouter embeddings)
uma search "how do we structure code" --mode hybrid
uma search "reqwest" --mode keyword
uma search "error handling conventions" --mode semantic

# Vectorize all missing embeddings
uma search --vectorize

# Store a project decision
uma write --type decision --title "Use SQLite for BM25 Index" --desc "Index location" --body "Store the FTS5 index in the user profile." --tags search,sqlite

# Store a global user preference
uma write --scope global --type preference --title "Preferred UI Library" --body "Always use Ratatui for terminal interfaces."

# Supersede a fact with a revised revision (old kept as deprecated)
uma supersede <old-id> --title "New revision" --desc "One-line summary" --body "..."

# Time-travel: what was true on a given date
uma search "config storage" --as-of "2026-01-15T00:00:00Z"

# Include superseded history (shows [DEPRECATED])
uma search "config storage" --include-deprecated

# Read, list
uma read <ULID>
uma list --scope global
uma list --type decision

# Rewrite legacy markdown files into the OKF v0.2 layout
uma migrate --dry-run
uma migrate --reindex

# Rebuild the FTS index after a schema change
uma search "anything" --reindex
```

Bodies may be piped via stdin:

```bash
uma write --type decision --title "Long body" <<'EOF'
### Context
...
EOF
```
