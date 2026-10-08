---
id: 01M4DDVQY350QS7FZ5807WFKWY
scope: "project:ai-memory"
type: decision
title: Defer automatic context injection (Slice S3)
description: Prioritize on-demand search plus operator-approved writes over auto-injecting memories
tags:
  - recall
  - injection
  - s3
  - roadmap
  - context
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:35:10.787673700+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:35:10.787675300+00:00"
since: "2026-10-08T09:35:10.787675300+00:00"
---
### Context
Slice S3 would auto-surface top-N memories into the model's context each turn. Deciding what to inject is unreliable: relevance is query-dependent, and injecting stale or off-topic facts adds noise and can push the model toward acting on wrong information. It also spends context budget on every turn.

### Decision
Slice S3 (Auto-Recall & Context Injection) is ON HOLD / `[?]`, not cancelled. UMA favors:
- explicit, on-demand retrieval (`uma_search`, hybrid mode), and
- operator-approved writes via the approval modal,
so the agent pulls what it needs and the human gates what is stored.

### Consequences
- No silent context pollution; token cost stays predictable.
- Revisit only with a high-precision selector and an explicit opt-in switch.