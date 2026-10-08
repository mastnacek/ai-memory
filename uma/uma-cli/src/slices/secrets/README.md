# secrets — the credential gate

## Command

`uma secrets scan <TEXT> [--json] [--stdin]`

Scans text for credentials and injection signatures, printing one line per
finding. Block-severity findings (credentials, invisible unicode) make the
write and supersede slices refuse the fact; warning-severity findings
(injection signatures) are surfaced for the operator to judge.

## Why it exists

Memory is the most dangerous place a credential can end up: facts are
re-injected into every future session, exported, committed to git, and pushed
to remotes. Ported from `@pify/memory`'s secret gate (which credits
`pi-hermes-memory` for the idea), with two UMA changes: provider patterns are
anchored for precision and pass through a placeholder filter (doc examples,
env-var names, and templates never block), and a UMA-local env-literal pass
catches prefix-less internal keys — a value the process environment holds as
a secret-shaped variable is a credential by definition.

## Invariant

**Detection is deterministic, offline, and fail-closed.** No network, no
model — `cargo test` exercises the gate exactly as production does. Findings
never print a full credential (masked previews only). The scanner refuses on
block findings wherever text enters memory: `uma write`, `uma supersede`, and
the Pi proposal path scan before the approval modal.