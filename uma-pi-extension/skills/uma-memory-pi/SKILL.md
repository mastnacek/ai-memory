---
name: uma-memory-pi
description: Use UMA memory from the Pi agent — native uma_write/read/list/search/supersede tools, the approval modal, and /uma commands. Load this whenever you capture or retrieve project decisions and preferences in Pi.
---

# UMA Memory Skill (Pi agent)

This is the **Pi-specific** companion to the harness-agnostic UMA skill. Read the general skill
— the `uma-memory` skill, canonical at the repo root (`skills/uma-memory/SKILL.md`) and shipped
alongside this file in the Pi package — for the portable material: capture triggers, quality
standards, the OKF v0.2 frontmatter table, lifecycle semantics, and the full CLI reference.
**This file adds only what is specific to driving UMA from Pi.**

> **Do not merge the two skills.** The general file must stay usable by agents that have no Pi
> tools; this file may assume Pi's tool names, approval UI, and slash commands.

---

## 1. Prefer the Pi tools over the CLI

In Pi, call the **tools** — they share the same store as the CLI and add the approval workflow:

| Tool | Purpose |
| :--- | :--- |
| `uma_search` | Hybrid BM25 + semantic search. Args: `query`, `mode` (`keyword`\|`semantic`\|`hybrid`), `scope`, `type`, `includeDeprecated`, `asOf`, `limit`. |
| `uma_write` | Store a new fact. Args: `title`, `body`, `type`, `scope`, `tags`, `description`, `template`. Opens the approval modal. |
| `uma_supersede` | Replace a fact with a revised revision. Args: `oldId`, `title`, `body`, `description`, `type`, `scope`, `tags`, `template`. The predecessor is kept as `status: deprecated` and chained via `supersedes`. |
| `uma_consolidate` | Review memory for near-duplicate facts (proposed merges) and opposing facts (contradiction flags). Read-only, ungated. Apply a proposal with `uma_supersede` or `uma_write`. |
| `uma_skill_invoke` | Expand a stored `skill` template into a concrete command. Args: `name`, `set` (`['tag=v1']`), `scope`. Read-only, ungated. Returns **text only** — run the command yourself. |
| `uma_list` | List memories by `scope` and `type`. |
| `uma_read` | Read one fact by `id` (ULID). |

The `uma` CLI is still available in `bash` (and is the only path in non-interactive runs), but
inside an interactive Pi session the tools are preferred because they route through approval.

### Skills (procedural memory)

A `skill` fact carries a `template` — a command with `{{placeholder}}` slots. Author one when you
worked out a non-obvious command or sequence you would otherwise re-derive next session: call
`uma_write` with `type: "skill"` plus `template`, and it goes through the normal approval modal
(there is no separate gated tool for this).

**`uma_skill_invoke` expands a template and returns text. It never executes anything.** Run the
returned command yourself with `bash`, so your own command approval still applies. Never treat
UMA as an execution primitive. If the expansion reports missing placeholders, supply them — they
are left visible rather than silently blanked.

To revise a skill, use `uma_supersede`; the template is inherited unless you pass a new one.

---

## 2. Approval modal (what the operator sees)

Unless auto-approval is on, `uma_write`/`uma_supersede` open a review window first. The operator can:

- **[Enter]** approve and save,
- **[e]** edit the title, **[b]** edit the body, **[t]** edit the tags,
- **[s]** toggle scope (`project` ⇄ `global`),
- **[Esc]** reject — nothing is written.

If the operator edits the proposal, the edited version is what gets stored. If they reject, the
tool returns `details.rejected = true` and **you must not retry the write** — treat it as a
deliberate decision and move on.

---

## 3. Approval gate (fail-closed)

A `tool_call` hook (`src/hooks/approval_gate.ts`) guards every memory-mutating tool
(`uma_write`, `uma_supersede`, `uma_consolidate`):

- **Interactive TUI** + auto-approve off → allowed, modal opens.
- **Interactive TUI** + auto-approve on → allowed, writes directly.
- **No interactive UI** (`pi -p`, RPC, JSON, or a nested `codemode` call) + auto-approve off →
  **blocked** with an actionable reason.

Consequences for you:

- You **cannot** write memory from a non-interactive or scripted context unless the operator has
  enabled auto-approval. Do not attempt workarounds; surface the block reason instead.
- When a write is blocked, tell the operator to run interactively, or to enable
  `/uma auto-approve on`.
- Do not call the `uma` CLI to deliberately bypass the gate — that defeats the operator's consent
  and writes without review.

---

## 4. Slash commands

```text
/uma search <query>
/uma list [global]
/uma read <id>
/uma reindex
/uma lang cs|en [--global]
/uma auto-approve on|off [--global]
```

`auto-approve on` skips the modal for trusted bulk work; `off` (default) keeps operator review.
Settings use the normal cascade: without `--global` they are written to `<cwd>/.pi/uma.json`;
with it, to `~/.pi/agent/uma.json`.

---

## 5. Reload rule

**After modifying any UMA Pi extension code or resources, the operator must run `/reload` before
the new behaviour is exercised.** Never invoke a freshly modified plugin tool autonomously without
that reload — the running session still has the old module graph. When unsure whether a reload
happened, say so and wait rather than triggering the tool.

---

## 6. Working with the operator

- **Propose, do not impose.** For anything worth remembering, call `uma_write` so the operator can
  approve, edit, or reject it. Do not pre-approve on their behalf.
- **Check before writing** with `uma_search` to avoid duplicating an existing fact; prefer
  `uma_supersede` when revising one.
- **Search on demand.** Do not assume memories were injected into your context; retrieve
  explicitly when a task depends on prior decisions or preferences.
