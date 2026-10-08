# `timeline` slice

**Command:** `uma timeline [--id <ULID>] [-s <scope>] [-t <type>] [--all] [--json]`
**Depends on:** `uma-core::timeline`

## What it does
Reconstructs supersession history. Because each revision records `supersedes`, the facts form chains rather than a flat list; this slice walks those links and prints each chain oldest revision first, newest chain at the top.

- No `--id`: every chain that has more than one revision.
- `--id <ULID>`: only the chain containing that fact.
- `--all`: also show single-revision facts (which have no history).
- `--json`: machine-readable chains, including each revision's `since`/`until`.

## Why it exists
Supersession keeps memory honest only if the history is *readable*. Without this, "what did this used to say, and when did it change" is answerable only by grepping `supersedes` fields by hand. It is the temporal counterpart to `search --as-of`: that answers what was true at a moment, this shows how it got there.

## Invariant
- **Read-only**, and it always loads deprecated facts — they *are* the history. Filtering them out would leave every chain with one visible step.
- A chain is walked with a visited guard, so hand-edited frontmatter describing a cycle cannot hang the command.
- An orphaned revision (whose predecessor is absent) becomes its own root rather than being dropped or mis-linked.
