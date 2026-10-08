# `scopes` slice

**Command:** `uma scopes [--json]`
**Depends on:** `uma-core::store::lookup` (read-only index inspection)

## What it does

Lists every scope the central index knows about, with a fact count per scope — `global` and each `project:<name>` — largest first. `--json` emits the same list for tool consumers.

## Why it exists

Project memory is deliberately isolated: `list`, `read`, and unscoped `search` only see the current project plus global. Isolation is right, but it made cross-project recall *unreachable* — an agent in one project could not even name another project's scope to query with `--scope ai-memory`. Discovered by practical use: a fresh agent in a new project saw only global memory and had no way to learn that `project:ai-memory` existed.

This is the discovery primitive: `uma scopes` → `uma search "…" --scope <name>` → `uma read <id>`. Together they form a complete cross-project recall chain.

## Invariant

- **Read-only.** The index is opened through the same read-only inspection path the doctor uses; discovery must never mutate what it measures.
- Counts come from the **index**, which is a rebuildable cache — a scope listed here is only as current as the last reindex. The Markdown files remain the source of truth.
- Isolation is preserved: nothing here *reads* another project's facts, only the fact that the scope exists and how large it is.