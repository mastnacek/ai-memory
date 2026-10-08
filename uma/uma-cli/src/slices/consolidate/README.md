# `consolidate` slice

**Command:** `uma consolidate [-s <scope>] [-t <type>] [--threshold] [--title-weight] [--include-deprecated] [--dry-run] [--json]`
**Depends on:** `uma-core::{consolidate, similarity}`

## What it does
Scans active facts and reports two things:
- **Proposed merges** — groups of facts whose titles and bodies overlap. Pairs scoring above `--threshold` (blended 60% title / 40% body by default) are joined into connected components, and each component of two or more becomes one proposal.
- **Contradictions** — pairs that are wording-similar but carry opposite polarity, read from negation markers in the *title*.

It prints the report and modifies nothing. `--json` emits the same report for tool consumers.

## Why it exists
Supersession keeps memory consistent only when somebody notices the duplication. Left alone, a store accumulates restatements ("use pnpm" written twice) and live contradictions, and a future agent cannot tell which statement wins. This slice is the detection half of that job. Resolution deliberately stays outside it.

## Invariant
- **Read-only.** It must never write, supersede or reindex — applying a proposal belongs to `supersede` / `write`, which are the slices that carry consent. This is why `uma_consolidate` is not behind the approval gate.
- **Recall over precision.** A false proposal costs one glance; a missed duplicate silently rots into two competing truths. Polarity is read from titles only, because bodies contain comparative asides ("use pnpm, not npm") that would otherwise register as false conflicts.
