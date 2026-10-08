---
id: 01M4EP264QYX1FA23JQKT3GFQM
scope: "project:ai-memory"
type: decision
title: Auto-blocking is reserved for deterministic contracts; probabilistic verdicts may only warn
tags:
  - immune-interceptor
  - consent
  - contracts
  - roadmap
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T21:17:44.983583900+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T21:17:44.983584100+00:00"
since: "2026-10-08T21:17:44.983584200+00:00"
---
## Context

The immune interceptor (proposal 01) checks memory rules and the pain score before file-mutating tool calls land. A probabilistic verdict (Jev/router, confidence != 1.0) must never silently veto agent work — that would invert the fail-closed consent model.

## Rule

The interceptor ships warn-only: hazards surface as UI warnings, the operator decides. Auto-block may only be introduced for contract-backed rules (proposal 03: deterministic ast-grep contracts with a passing test proving the rule), never for model-judged violations.

## Consequences

- Every warning is advisory; agent work can never hang on the interceptor (bounded timeouts, fail-silent).
- Block-mode is gated on P03a landing, not on judge confidence thresholds.