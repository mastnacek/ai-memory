# `uma_search` slice (Pi)

**Tool:** `uma_search`

## What it does
Registers `uma_search` with `query`, `mode` (keyword / semantic / hybrid), `scope`, `type`, `limit`, `includeDeprecated` and `asOf`, delegating to `uma search`.

## Why it exists
This is the on-demand recall path that the whole design prefers over automatic context injection (S3, on hold). `asOf` gives the agent point-in-time recall, so it can ask what was believed *before* a decision changed instead of assuming today's fact was always true.

## Invariant
- Read-only.
- Deprecated facts stay hidden unless `includeDeprecated` is explicitly set.
