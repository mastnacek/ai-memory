# `list` slice

**Command:** `uma list [-s <scope>] [-t <type>] [--include-deprecated]`
**Depends on:** `uma-core::store::list`, `shared::format`

## What it does
Enumerates facts in a scope, optionally filtered by type, printing one line per fact. Deprecated and otherwise inactive facts are hidden unless `--include-deprecated` is passed, in which case they are badged `[DEPRECATED]` / `[DRAFT]`.

## Why it exists
Cheap orientation without a query — "what do I already know about this project?". Hiding deprecated facts by default is the point: printing a superseded rule beside its replacement invites an agent to act on the stale one, which is the exact hazard supersession exists to remove.

## Invariant
- Applies the same active/deprecated contract as `search`.
- Reports how many facts it hid, so silence is never mistaken for "nothing stored".
