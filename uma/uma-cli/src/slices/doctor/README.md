# `doctor` slice

**Command:** `uma doctor [--json] [--strict]`
**Depends on:** `uma-core::{store, health}`

## What it does
A read-only health report over the store and its rebuildable index cache. It checks:
- both store roots exist, and how many Markdown documents are on disk
- the index database exists (size included), and its recorded schema version matches this build
- **index coverage** — indexed rows versus files on disk
- embeddings: absent, partially filled, or complete
- **orphan embeddings** and **stale rows** (rows pointing at deleted files)

Every non-ok finding prints the exact command that would fix it. `--strict` exits non-zero when anything fails.

## Why it exists
The index is a *cache*, and caches drift: a deleted file, a store removed without pruning, an older build's schema. Drift is invisible until search silently returns results for files that no longer exist. This makes the drift legible and names the remedy.

It also counts facts that are still `stable` but past their `stale_after` date. Staleness is a *store* concern rather than an index one — a claim whose validity window has closed needs re-verification whether or not it is indexed — and it is deliberately distinct from deprecation: replaced is not the same as possibly expired.

## Invariant
- **Reports; never repairs.** It opens the index with `SQLITE_OPEN_READ_ONLY`, because `Indexer::open` deliberately drops and recreates `facts_fts` on a schema mismatch — correct for normal use, wrong for a diagnosis. A health check must never mutate what it inspects.
- Remedies are printed as commands for the operator to run deliberately, so a diagnosis can never become an unrequested mutation.
