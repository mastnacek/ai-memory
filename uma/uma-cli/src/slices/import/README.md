# import — session-history to memory candidates

## Command

`uma import sessions [--project TEXT] [--source pi|claude|all] [--session ID|PATH] [--limit N] [--max-per-session N] [--json]`

Extracts memory candidates from the pi / Claude Code session stores and prints
a review report: suggested type, suggested title, the source quote, session
provenance, and the exact `uma write` command (with `--since <session date>`)
that would propose it.

## Why it exists

Months of agent conversations contain real decisions and corrections, but the
sessions themselves are deleted with their projects (`mozek_rust` had 26
sessions across two machines before its directory vanished). This slice makes
that history mineable without making it dangerous.

## Invariant

**This slice proposes; it never writes.** It contains no store mutation at
all — the only path from a candidate into memory is the gated `uma_write`/
`uma_supersede` tools, where the operator sees and approves each fact.
Extraction is deterministic heuristics (language markers, dedup, provenance);
understanding, translation (Czech→English), and quality judgment are the
reviewing agent's job, done one proposal at a time. `--since` preserves the
session's date so an imported claim does not masquerade as being made today.