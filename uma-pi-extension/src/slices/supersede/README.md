# `uma_supersede` slice (Pi)

**Tool:** `uma_supersede`
**Guarded by:** the fail-closed approval gate

## What it does
Registers `uma_supersede`. It first reads the predecessor with `uma read <old-id> --json` to inherit its real type, scope and tags, then shows the same approval modal as `uma_write` — including a `Supersedes: <ULID>` line so the reviewer can see what is being replaced — and finally calls `uma supersede` with the approved values.

## Why it exists
Revising memory must be at least as visible as creating it. Without the predecessor lookup the modal would display guesswork instead of the fact's actual metadata; without the modal a supersession would rewrite live memory silently, which is exactly what this slice did before it was fixed.

Like `uma_write`, this tool accepts `template` and `staleAfter`. Omitted values are inherited from the predecessor - including a `stale_after` that has already passed, which the modal shows so the operator can pass a fresh date when re-verifying.

## Invariant
- Never supersede after a rejection.
- The predecessor is only ever deprecated, never deleted.
