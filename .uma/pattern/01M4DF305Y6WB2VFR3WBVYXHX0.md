---
id: 01M4DF305Y6WB2VFR3WBVYXHX0
scope: "project:ai-memory"
type: pattern
title: Keep hook logic pure; reserve the subscription for the composition root
tags:
  - hooks
  - testing
  - vsa
  - lifecycle
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:56:37.182578900+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:56:37.182581600+00:00"
since: "2026-10-08T09:56:37.182581600+00:00"
---
### Context
Hook logic entangled with `pi.on(...)` registration cannot be unit-tested, and ad-hoc subscriptions leak into later sessions.

### Pattern
- `evaluateApprovalGate(event, ctx, state)` is a **pure function** returning `{ block: true, reason }` or `undefined`. It is unit-tested directly — no engine, no UI, no I/O (6 cases in `test/approval_gate.test.ts`).
- The composition root (`index.ts`) owns the subscription: `track(pi.on("tool_call", ...))`, where `track` pushes the returned unsubscribe into `state.unsubscribers`.
- `session_shutdown` drains `state.unsubscribers`, so a reload or session end leaves no live hooks behind.

### Consequences
- The approval decision is testable without a running editor.
- Lifecycle cleanup becomes a structural invariant: subscriptions are always created through `track`, never stored ad hoc.
- Slice code stays free of engine registration, which matches slice isolation in VSA.