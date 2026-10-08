# `supersede` slice

**Command:** `uma supersede <old-id> --title "<t>" [--body] [--desc] [--type] [--scope] [--tags]`
**Depends on:** `uma-core::store::{find_by_id, supersede}`

## What it does
Writes a new fact and retires its predecessor in one operation: the old fact gains `status: deprecated` plus an `until` timestamp, and the new fact records `supersedes: <old-id>`. Omitted type/scope/tags **and `stale_after`** are inherited from the predecessor.

Inheriting `stale_after` is deliberate but has a sharp edge: re-verifying a claim means passing a *new* `--stale-after`. Inherit one that has already passed and the revision is stale at birth — visible as `[STALE]`, which is the honest signal that verification is still owed.

## Why it exists
Memory must be able to change without losing its history. The alternative — adding a second, contradicting fact — leaves two live truths and no way for a future agent to tell which won. Supersession makes the replacement explicit and the resolution auditable, and `--as-of` can still reconstruct what was believed earlier.

## Invariant
- The predecessor is deprecated, never deleted.
- The chain stays unbroken: every revision points at exactly one predecessor.
