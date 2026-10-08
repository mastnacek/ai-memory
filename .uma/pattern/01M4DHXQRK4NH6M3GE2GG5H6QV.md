---
id: 01M4DHXQRK4NH6M3GE2GG5H6QV
scope: "project:ai-memory"
type: pattern
title: A health check must open the cache read-only
tags:
  - cache
  - doctor
  - safety
  - sqlite
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T10:46:10.451127900+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T10:46:10.451131300+00:00"
since: "2026-10-08T10:46:10.451131300+00:00"
---
### Context
`Indexer::open` is **not** a safe way to *look* at the index. When the recorded schema version or the actual column layout disagrees with the build, it deliberately drops and recreates `facts_fts`. That is correct for normal use — the index is a rebuildable cache — and wrong for inspection: a diagnosis would silently repair what it was supposed to report on, destroying the evidence it was asked to find.

### Pattern
Read-only inspection is a capability distinct from normal access:
- `uma-core/src/health.rs` opens the database with `SQLITE_OPEN_READ_ONLY` and never migrates.
- The SQL lives in the kernel beside the schema it queries, so the read-only guarantee is enforced in one place, and CLI slices need no database dependency of their own.
- `uma doctor` **reports; never repairs.** Every non-ok finding prints the exact command that would fix it, and the operator runs it deliberately. `--strict` only decides whether findings become a non-zero exit.

### Consequences
- A health check cannot change what it measures, so results are trustworthy and repeatable.
- A diagnosis cannot mask the drift it just found.
- Remedies stay visible as commands, keeping a diagnostic from becoming an unrequested mutation — the same principle as the approval gate and the read-only consolidator.
- It proved its worth immediately: the first run reported *38 indexed rows vs 26 files on disk* and *12 stale rows*, leftover pollution from test stores written before canonicality gating existed.