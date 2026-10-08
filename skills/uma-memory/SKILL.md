---
name: uma-memory
description: Store, search, and recall structured project decisions, user preferences, architectural patterns, and reusable skills using the UMA memory engine.
---

# UMA Memory Skill

This skill guides coding agents on when, why, and how to capture, organize, and retrieve structured long-term memories using the **UMA (Universal Memory Architecture)** engine.

---

## 1. When to Capture Memory

Agents should actively record memory into UMA under the following triggers:

| Event Trigger | Fact Type | Scope | Example |
| :--- | :--- | :--- | :--- |
| **Architectural Decision** | `decision` | `project` | Choosing Vertical Slice Architecture, database selection, API protocols. |
| **User Preference** | `preference` | `global` | "Always use Rust & Ratatui for TUI tools", "Prefer concise explanations". |
| **Codebase Pattern** | `pattern` | `project` | Error handling conventions, module isolation rules, test layout. |
| **Correction / Mistake Fix** | `correction` | `project` / `global` | Correcting a false assumption, fixing a recurring bug pattern. |
| **Environmental Fact** | `fact` | `project` | API endpoint URLs, build requirements, port allocations. |
| **Procedural How-To** | `skill` | `project` / `global` | Step-by-step commands to build, package, or deploy the artifact. |

### What NOT to Store:
- Ephemeral chat chatter or greetings.
- Raw temporary code dumps that will change in the next turn.
- Obvious information already fully documented in standard files (e.g., standard `Cargo.toml` dependencies).

---

## 2. Memory Quality Standards

When calling `uma_write` or executing `uma write`:

1. **Atomic & Focused**: One decision or fact per memory entry.
2. **Clear Actionable Title**:
   - Good: `Adopt Vertical Slice Architecture for CLI modules`
   - Bad: `Architecture notes`
3. **Structured Markdown Body**:
   - **Context**: Why this was decided or what problem it solves.
   - **Decision / Rule**: The exact convention or constraint to follow.
   - **Consequences**: What to do and what to avoid.
4. **Relevant Tags**: Lowercase ASCII keywords (`tags: ["vsa", "architecture", "rust"]`).

---

## 3. Tool Reference (Pi Agent Tools)

### `uma_search`
Search memory facts using hybrid BM25 + semantic vector matching (RRF):
```json
{
  "query": "how do we isolate slices in VSA architecture?",
  "mode": "hybrid",
  "limit": 5
}
```

### `uma_write`
Store a structured fact (in interactive mode, prompts the user via a TUI modal review window):
```json
{
  "type": "decision",
  "scope": "project",
  "title": "Adopt Vertical Slice Architecture",
  "body": "All CLI commands must be isolated in `src/slices/` without cross-slice imports.",
  "tags": ["vsa", "architecture", "rust"]
}
```

### `uma_list`
Query available memories by scope and type:
```json
{
  "scope": "project",
  "type": "decision"
}
```

### `uma_read`
Retrieve the full body and metadata of a specific fact:
```json
{
  "id": "01M4D6K8J6QGDFC0Y11FW45R87"
}
```

### `uma_supersede`
Replace a fact with a revised revision. The predecessor is kept (never deleted) as `status: deprecated` with an `until` timestamp, and the new fact chains back to it via `supersedes`:
```json
{
  "oldId": "01M4D6K8J6QGDFC0Y11FW45R87",
  "title": "Use TOML for config storage",
  "description": "Config persistence format (revised)",
  "body": "We switched from JSON to TOML for config."
}
```

---

## 5. OKF v0.2 Frontmatter

UMA facts are valid **Open Knowledge Format v0.2** documents (Markdown + YAML frontmatter),
extended with UMA-specific keys. Alignment:

| Key | Source | Meaning |
| :--- | :--- | :--- |
| `type` | OKF | Concept type (`decision`, `preference`, `pattern`, ...) |
| `title` | OKF | Human-readable name |
| `description` | OKF | One-line summary used for previews/snippets |
| `tags` | OKF | Cross-cutting categorization |
| `status` | OKF | `stable` (default) \| `deprecated` \| `draft` |
| `generated` | OKF | `{ by: pi-agent/1.1, at: <iso8601> }` — who produced it |
| `verified` | OKF | `[{ by: human:operator, at: <iso8601> }]` — derived trust tier |
| `since` / `until` / `stale_after` | OKF lifecycle | Validity window and staleness instant |
| `id` | UMA | ULID identifier |
| `scope` | UMA | `global` or `project:<name>` |
| `supersedes` | UMA | ULID of the replaced predecessor fact |

Trust tier is derived from `verified`: no `verified` → unverified; only non-`human:` actors
→ machine-confirmed; a `human:<id>` actor → human-reviewed.

---

## 4. CLI Reference

```bash
# Hybrid search (BM25 + Semantic OpenRouter embeddings)
uma search "how do we structure code" --mode hybrid
uma search "reqwest" --mode keyword
uma search "error handling conventions" --mode semantic

# Vectorize all missing embeddings
uma search --vectorize

# Supersede a fact with a revised revision (old kept as deprecated)
uma supersede <old-id> --title "New revision" --desc "One-line summary" --body "..."

# Time-travel: what was true on a given date
uma search "config storage" --as-of "2026-01-15T00:00:00Z"

# Include superseded history (shows [DEPRECATED])
uma search "config storage" --include-deprecated

# Rewrite legacy markdown files into the OKF v0.2 layout
uma migrate --dry-run
uma migrate --reindex

# Rebuild the FTS index after a schema change
uma search "anything" --reindex

# Store project decision
uma write --type decision --title "Use SQLite for BM25 Index" --body "Store FTS5 index in user profile." --tags search,sqlite

# Store global user preference
uma write --scope global --type preference --title "Preferred UI Library" --body "Always use Ratatui for terminal interfaces."

# List facts
uma list --scope global
uma list --type decision

# Read fact by ID
uma read <ULID>
```
