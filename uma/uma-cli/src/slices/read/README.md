# `read` slice

**Command:** `uma read <id> [--json]`
**Depends on:** `uma-core::{store::find_by_id, serialization}`

## What it does
Resolves a ULID to a fact, searching every known scope root, then prints it as a human-readable block or as JSON (`--json`).

## Why it exists
Retrieval for when the identifier is already known. Search is for discovering an id; read is for acting on one deterministically — no ranking, no ambiguity.

`--json` exists for machine consumers: it is how the Pi `uma_supersede` tool fetches a predecessor's real type, scope and tags to prefill its approval modal.

## Invariant
- Never mutates anything, including the index.
- A missing id is an error, not an empty result.
