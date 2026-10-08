# `uma_skill_invoke` slice (Pi)

**Tool:** `uma_skill_invoke`
**Not gated:** read-only — it expands a template, it never writes *or executes*

## What it does
Registers `uma_skill_invoke`, which calls `uma skill invoke <name> --json` and returns the expanded command text plus any missing or unused placeholders.

## Why it exists
Procedural memory is only useful if the agent can turn a stored template into a runnable command without re-deriving it. The tool stops at *text* on purpose: the agent then runs the command through its own shell tool, where the harness's existing command approval already governs execution.

That split matters. If this tool executed the template, UMA would become a code-execution surface reachable through a *memory* API, and it would bypass the approval the shell tool already provides. Keeping expansion pure means the only thing that changes is who presses go.

## Invariant
- Returns text only, never runs anything. JSON from the CLI carries `"executed": false`.
- Must not be added to the approval gate: gating a pure string expansion would make approval routine.
- Skill *creation* goes through `uma_write` (type `skill` + `template`), so it stays behind the gated path. This deliberately avoids introducing a new mutating tool.
