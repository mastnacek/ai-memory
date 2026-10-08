---
id: 01M4DFRFT86DES64VEPQF5WXCF
scope: "project:ai-memory"
type: decision
title: Memory writes fail closed when no approval UI exists
description: tool_call hook blocks mutations without a TUI or auto-approve; consolidator is read-only and ungated
tags:
  - approval
  - safety
  - hooks
  - pi
status: stable
supersedes: 01M4DF2HJGWMB01JR5B8ZKHNXM
generated:
  by: pi-agent/1.1
  at: "2026-10-08T10:08:21.320348600+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T10:08:21.320348800+00:00"
since: "2026-10-08T10:08:21.321138400+00:00"
---
### Context
An earlier version of this decision listed `uma_consolidate` among the tools guarded by the `tool_call` hook. That was written before the slice existed, from the PRD's planned tool list.

### Decision
The gate guards exactly the tools that mutate the store: **`uma_write` and `uma_supersede`**. A `tool_call` hook (`uma-pi-extension/src/hooks/approval_gate.ts`) is fail-closed: a mutation is allowed only when an interactive TUI approval UI exists (the tool then shows its modal) **or** the operator enabled `autoApprove`. It is blocked in `pi -p`, RPC, JSON and nested `codemode` calls.

### Consequences
- `uma_consolidate` is deliberately **not** gated. It only proposes merges and contradictions and never writes; gating a read would make approval routine and therefore meaningless.
- Adding a future mutating tool means adding one entry to `MEMORY_MUTATING_TOOLS` — the guard is centralized, never duplicated per tool.
- The gated set is locked by a test asserting it is exactly `{uma_write, uma_supersede}`.