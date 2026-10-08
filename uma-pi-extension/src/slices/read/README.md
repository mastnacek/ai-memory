# `uma_read` slice (Pi)

**Tool:** `uma_read`

## What it does
Registers `uma_read`, which resolves a ULID via `uma read <id>` and returns the fact text to the model.

## Why it exists
Deterministic retrieval when an id is already known — typically one returned by `uma_search` or `uma_list`. Read-only, so it is deliberately *not* behind the approval gate: reading memory is not a mutation, and gating it would train the operator to approve reflexively.

## Invariant
- Read-only: never writes and never touches the index.
