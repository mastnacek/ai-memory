# `migrate` slice

**Command:** `uma migrate [--dry-run] [--reindex]`
**Depends on:** `uma-core::{store, serialization, indexer}`

## What it does
Rewrites legacy fact files into the OKF v0.2 frontmatter layout, backfilling keys that did not previously exist (`status`, `generated`, `verified`, `description`). `--dry-run` reports what would change; `--reindex` rebuilds the search index afterwards.

## Why it exists
Facts written before OKF v0.2 was adopted are schema-incomplete. Without a forward migration, every reader would need a compatibility branch for "old" facts, and those branches would live forever. This slice pays the cost once so the rest of the system can assume a single schema.

## Invariant
- Idempotent: a second run must change nothing.
- Never silently drops data it does not understand — unknown keys are preserved.
