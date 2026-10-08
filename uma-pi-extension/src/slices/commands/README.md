# `commands` slice (Pi)

**Command:** `/uma <search|list|read|reindex|lang|auto-approve>`
**Files:** `index.ts` (dispatcher), `complete.ts` (tab completion)

## What it does
Registers the single `/uma` slash command and dispatches on its first token: `search`, `list`, `read`, `reindex`, `lang` and `auto-approve`. `complete.ts` supplies argument completion — subcommand names, `list project|global`, `lang cs|en` and `auto-approve on|off`.

## Why it exists
An operator-facing surface for memory that does not require the model to be in the loop. That matters most for `lang` and `auto-approve`: both are operator-owned settings, and `auto-approve` is the *only* escape hatch from the approval gate, so it must be changeable by a human and never by the agent.

## Invariant
- No memory writes. The only mutations are the config settings and `reindex`, which rebuilds the rebuildable index cache.
- The agent cannot reach `auto-approve` — it exists only as a slash command.
