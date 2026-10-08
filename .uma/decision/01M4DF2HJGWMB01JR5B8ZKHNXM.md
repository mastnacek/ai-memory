---
id: 01M4DF2HJGWMB01JR5B8ZKHNXM
scope: "project:ai-memory"
type: decision
title: Memory writes fail closed when no approval UI exists
tags:
  - approval
  - safety
  - hooks
  - pi
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:56:22.224725500+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:56:22.224728300+00:00"
since: "2026-10-08T09:56:22.224728300+00:00"
---
### Context
Memory-mutating tools must not be able to write without operator consent. A check inside each tool is not enough: nested `codemode` calls, RPC/JSON modes, and `pi -p` print mode can all reach a tool without a usable approval UI.

### Decision
A `tool_call` hook (`uma-pi-extension/src/hooks/approval_gate.ts`) guards every memory-mutating tool (`uma_write`, `uma_supersede`, `uma_consolidate`), registered from the composition root via `track(pi.on("tool_call", ...))`. It is **fail-closed**:
- Allowed only when an interactive approval UI exists (Pi TUI) — the tool then shows its modal — **or** the operator enabled `autoApprove`.
- Blocked in `pi -p`, RPC, JSON, and nested `codemode` calls.

The hook runs before execution and can return `{ block: true, reason }`. The reason is surfaced to the model and must tell the agent not to retry via another path.

### Consequences
- Verified end-to-end: a print-mode `uma_write` was blocked and nothing was persisted.
- Operator consent cannot be bypassed by changing invocation mode.
- `autoApprove` is the single explicit, operator-owned escape hatch.