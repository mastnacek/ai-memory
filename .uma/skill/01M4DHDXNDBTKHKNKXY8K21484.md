---
id: 01M4DHDXNDBTKHKNKXY8K21484
scope: "project:ai-memory"
type: skill
title: verify-uma-changes
description: "Full verification gate: cargo test, release build, extension typecheck+tests, reload"
template: "cd {{repo}}/uma && cargo test && cargo build --release && cd {{repo}}/uma-pi-extension && npm run typecheck && npm run test"
tags:
  - verification
  - workflow
  - testing
status: stable
supersedes: 01M4DGZ8NWXV5YR18P4MNC6B29
generated:
  by: pi-agent/1.1
  at: "2026-10-08T10:37:32.205045300+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T10:37:32.205045400+00:00"
since: "2026-10-08T10:37:32.205831500+00:00"
---
### Context
Every UMA change needs the same verification sequence, and skipping part of it has already caused a miss. A release build can emit warnings while `cargo test` reports clean, because tests only compile `cfg(test)` code: an unused field warned in `cargo build --release` but not in `cargo test`, and AGENTS.md requires **zero** warnings.

This skill was first created without its `template` (the running extension predated the `template` parameter and stripped it), so `uma skill invoke` reported "no invocation template". This revision restores it.

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
- Step 4 is not theoretical: ignoring it silently produced this very broken skill.