---
id: 01M4DK2MWARK7RZS9ZZM8D0EC9
scope: "project:ai-memory"
type: pattern
title: "Degradation belongs in the return value, not a log line"
tags:
  - degradation
  - correctness
  - api
  - design
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T11:06:19.914890900+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T11:06:19.914895500+00:00"
since: "2026-10-08T11:06:19.914895600+00:00"
---
### Context
A fallback expressed as a log line (`eprintln!`, `tracing::warn!`) is a signal the caller can drop anywhere on the way to the user. UMA's hybrid search fell back to keyword-only when the embedding client was unavailable — the agent asked for hybrid ranking, silently got BM25, and the returned results were confidently wrong in a way the consumer could not see.

### Pattern
When an operation can run in a degraded mode, make the degradation **a value the caller must handle**, not a side effect:
- Return a result type carrying `degraded: Option<Degradation>` (e.g. `SearchOutcome`), not just the data.
- The degradation names the requested mode, the performed mode, the reason, and a remedy.
- **Degrade rather than fail** when the degraded mode still answers the question: compute the robust half first and keep it, so a failing optional half never discards results already obtained.
- **Fail loudly** when the caller explicitly asked for the fragile mode (`--mode semantic` errors), and **degrade visibly** when it asked for the robust one (`--mode hybrid` reports the fallback).

### Consequences
- A caller cannot be misled about what it received without deliberately ignoring the field.
- Every layer (CLI, MCP) surfaces the notice up front, so it survives to the human.
- Verified end-to-end: an invalid key makes hybrid return keyword results *and* report the exact HTTP error, while `--mode semantic` errors and `--mode keyword` is unaffected.