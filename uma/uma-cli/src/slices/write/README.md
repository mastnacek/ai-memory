# `write` slice

**Command:** `uma write -t <type> -T "<title>" [-b "<body>"] [--desc] [--tags] [--scope]`
**Depends on:** `uma-core::{store, serialization}`

## What it does
Creates one fact: assigns a ULID, builds the OKF v0.2 frontmatter, writes the Markdown file into the scope root (`.uma/<type>/` for a project, the global directory otherwise), then indexes it into SQLite FTS5 and embeds it for vector search when a key is available.

## Why it exists
It is the only entry point for new memory — every other slice reads what this one writes. It is guarded by the approval gate precisely because it mutates durable state.

## Invariant
- Write is atomic: one concept per fact, so search, supersession and consolidation can reason about facts individually.
- Indexing is best-effort. A failure there must never lose the fact file, which is the source of truth.
