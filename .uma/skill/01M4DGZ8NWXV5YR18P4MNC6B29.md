---
id: 01M4DGZ8NWXV5YR18P4MNC6B29
scope: "project:ai-memory"
type: skill
title: verify-uma-changes
tags:
  - verification
  - workflow
  - testing
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T10:29:31.964479900+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T10:29:31.964481800+00:00"
since: "2026-10-08T10:29:31.964481900+00:00"
---
### Context
Every UMA change needs the same verification sequence, and skipping part of it has already caused a miss. A release build can emit warnings while `cargo test` reports clean, because tests only compile `cfg(test)` code: an unused field warned in `cargo build --release` but not in `cargo test`, and AGENTS.md requires **zero** warnings.

### Procedure
Run from the repo root after any Rust or extension change:
1. `cd uma && cargo test` — must pass with 0 warnings
2. `cargo build --release` — a *separate* check; warnings that exist only outside `cfg(test)` show up here and nowhere else
3. `cd ../uma-pi-extension && npm run typecheck && npm run test`
4. If extension code changed, `/reload` before exercising tools in Pi — a running session keeps the old module graph loaded

For anything that writes memory, rehearse in a throwaway repo first (`mktemp -d && git init -q .`) so no test run can touch the real store.

### Consequences
- Template, type, and formatting regressions are caught before they are committed.
- Never claim a change is verified from `cargo test` alone — the release build is a distinct gate.
- Docs and roadmap edits are part of "done", not an afterthought.