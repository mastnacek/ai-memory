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

### `uma_write`
Store a structured fact:
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
Query available memories before designing new features:
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

---

## 4. CLI Reference

```bash
# Store project decision
uma write --type decision --title "Use SQLite for BM25 Index" --body "Store FTS5 index in .uma/index.db." --tags search,sqlite

# Store global user preference
uma write --scope global --type preference --title "Preferred UI Library" --body "Always use Ratatui for terminal interfaces."

# List facts
uma list --scope global
uma list --type decision

# Read fact by ID
uma read <ULID>
```
