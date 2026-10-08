# `uma_consolidate` slice (Pi)

**Tool:** `uma_consolidate`
**Not gated:** read-only — it proposes, it never writes

## What it does
Registers `uma_consolidate`, which calls `uma consolidate --json` and returns the proposal report (proposed merges and contradiction flags) to the model.

## Why it exists
The detection half of keeping memory consistent, surfaced where the agent can act on it. An agent that has just noticed two competing facts can ask for a full sweep instead of stumbling across duplicates one at a time.

Applying a proposal is deliberately **not** part of this tool: the agent reviews the report and then calls `uma_supersede` or `uma_write`, which are gated and therefore require the operator's approval. That split is what keeps a bulk cleanup from becoming a silent rewrite.

## Invariant
- Read-only, and the only mutating tools stay `uma_write` and `uma_supersede`.
- Must not be added to the approval gate: gating a read would train the operator to approve reflexively.
