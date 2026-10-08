# `uma_write` slice (Pi)

**Tool:** `uma_write`
**Guarded by:** the fail-closed approval gate

## What it does
Registers the `uma_write` tool. It collects title, body, type, scope and tags, shows the approval modal when an interactive TUI is available and `autoApprove` is off, then shells out to `uma write` with the approved values.

## Why it exists
The Pi-facing entry point for new memory. The modal is the consent step — without it the agent would write durable state silently. On rejection nothing is executed and `details.rejected` is set, so the model can see the write did not happen rather than assuming success.

A fact may also carry `template` (for `skill` facts) and `staleAfter` (the date after which the claim needs re-verification) - both shown in the modal so the reviewer sees exactly what will be stored.

## Invariant
- Never invoke the CLI after a rejection.
- With no interactive approval UI the gate blocks this tool before `execute` ever runs.
