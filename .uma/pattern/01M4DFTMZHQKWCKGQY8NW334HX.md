---
id: 01M4DFTMZHQKWCKGQY8NW334HX
scope: "project:ai-memory"
type: pattern
title: Every slice folder documents itself with a README
tags:
  - vsa
  - documentation
  - convention
  - slices
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T10:09:32.145167900+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T10:09:32.145169500+00:00"
since: "2026-10-08T10:09:32.145169500+00:00"
---
### Context
The PRD's acceptance criterion was "docs updated: `docs/slice-<N>.md`" — a central docs tree. Nothing was ever written there: **zero** slice docs existed for any slice, because a document separated from the code it describes drifts and is easy to forget.

### Pattern
Each slice is a **folder that carries its own documentation**:
- Rust: `uma-cli/src/slices/<feature>/mod.rs` + `README.md`
- Pi: `uma-pi-extension/src/slices/<feature>/index.ts` + `README.md`

Each README is concise (roughly 15-30 lines) and states **what the slice does, why it exists, and the invariant it must not break**, plus its command/tool name and its dependencies.

Slices are folders, not files: `slices/<x>.rs` was restructured into `slices/<x>/mod.rs`. In the Pi extension the former `slices/tools/` aggregator was removed so the composition root wires each slice directly instead of through an extra layer.

### Consequences
- A slice without its README is incomplete — AGENTS.md §2 and §5 now require the folder + README for every new slice.
- The docs acceptance criterion points at the colocated README rather than `docs/slice-<N>.md`; the PRD was updated accordingly.
- Rejected alternatives: sidecar `slices/<x>.md` files (puts prose inside the Rust module directory) and a central `docs/slices/` tree (drifts from the code and gets forgotten).