---
id: 01M4DF2R2ADPBG1E171QED5M72
scope: "project:ai-memory"
type: pattern
title: "Approval is a property of the tool path, never of the skill"
tags:
  - approval
  - cli
  - skills
  - consent
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:56:28.874683200+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:56:28.874685600+00:00"
since: "2026-10-08T09:56:28.874685700+00:00"
---
### Context
The approval modal lives in the Pi extension's tool implementations. The `uma` CLI has **no approval code at all** — `uma write` writes unconditionally. This causes a recurring confusion: it looks like the skill "enables" or "controls" approval, when it cannot.

### Pattern
Approval is a property of the **tool path**, never of a skill:
- A skill cannot grant, weaken, or implement approval — skills are instructions, not enforcement.
- The CLI is unconditional by design: it is the scriptable interface used by agents in other harnesses, so invoking it bypasses operator consent entirely.
- Therefore: never use the `uma` CLI to perform a memory write that the approval gate blocked. That defeats the consent contract.

### Consequences
- "The skill allowed the write" is never a correct explanation; the only gates are the hook, the modal, and `autoApprove`.
- An agent that must write in a non-interactive context needs `autoApprove`, not a CLI workaround.
- The CLI remains correct for automation and for direct human use, where consent is the human typing the command.