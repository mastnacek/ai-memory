---
id: 01M4DX0PW8BWEERKQ3XRQR1FYK
scope: "project:ai-memory"
type: pattern
title: Keep hook logic pure; reserve the subscription for the composition root
description: Pure evaluateApprovalGate + composition-root subscription drained on shutdown
tags:
  - hooks
  - testing
  - vsa
  - lifecycle
status: stable
supersedes: 01M4DF305Y6WB2VFR3WBVYXHX0
generated:
  by: pi-agent/1.1
  at: "2026-10-08T14:00:02.184767800+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T14:00:02.184767900+00:00"
since: "2026-10-08T14:00:02.185511700+00:00"
---
### Context
Hook logic entangled with `pi.on(...)` registration cannot be unit-tested, and ad-hoc subscriptions leak into later sessions.

### Pattern
- `evaluateApprovalGate(event, ctx, state)` is a **pure function** returning `{ block: true, reason }` or `undefined`. It is unit-tested directly — no engine, no UI, no I/O.
- The composition root (`index.ts`) owns the subscription: `track(pi.on("tool_call", ...))`, where `track` pushes the returned unsubscribe into `state.unsubscribers`.
- `session_shutdown` drains `state.unsubscribers`, so a reload or session end leaves no live hooks behind.

### Consequences
- The approval decision is testable without a running editor.
- Lifecycle cleanup becomes a structural invariant: subscriptions are always created through `track`, never stored ad hoc.
- Slice code stays free of engine registration, which matches slice isolation in VSA.
- **Facts and comments describe invariants, not volatile state.** An earlier revision of this fact cited a test count; the count changed the same day the fact was written. A number embedded in prose goes stale the moment the code moves — state the rule that stays true instead.