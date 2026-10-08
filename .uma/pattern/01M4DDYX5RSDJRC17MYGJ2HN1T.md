---
id: 01M4DDYX5RSDJRC17MYGJ2HN1T
scope: "project:ai-memory"
type: pattern
title: Side effects must be gated on store canonicality
description: "Ad-hoc Store::new paths must not write into the shared central index"
tags:
  - store
  - index
  - testing
  - side-effects
  - pattern
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:36:54.456699700+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:36:54.456701500+00:00"
since: "2026-10-08T09:36:54.456701500+00:00"
---
### Context
`Store::write` unconditionally opened the shared central index and inserted a row. Tests and any temporary `Store::new(tempdir)` therefore wrote their markdown to a temp folder but pushed the index row (and an embedding) into the production database. Result: 28 index rows against 9 real facts, 154 orphan embeddings, and `Note 1`/`Note 2` noise in semantic search — a hidden, growing corruption of the shared cache.

### Pattern
Gate shared-state side effects on whether the actor is entitled to touch that state. `Store::is_canonical()` returns true only for the global root or the current project's `.uma`; only those feed the central index. Ad-hoc stores write Markdown and nothing else.

### Consequences
- Tests are hermetic and fast (they no longer make network calls for embeddings).
- No orphan rows or embeddings can accumulate from temp/throwaway stores.
- A regression test asserts a non-canonical store leaves the central index untouched.