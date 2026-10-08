# immune — the warn-mode interceptor's decision slice

## Behavior

Pure, transport-free decision logic for the immune interceptor
(`src/hooks/immune_interceptor.ts`, which owns the pi `tool_call`
subscription): `extractEdit` reads write/edit tool inputs into
`{path, added}`, and `assessEdit` turns a pain verdict plus the L1 rules
into human-readable warnings — pain bands from the kernel's risk module,
and a rule warning when the edit shares ≥4 tokens with a rule and the rule
covers ≥25% of the edit (containment metric; Jaccard was deaf on long
rule bodies). Warn-only, never a block.

## Why it exists

The deterministic advisory half of `docs/proposals/01`: the operator sees
memory-relevant hazards before code lands, without trusting a probabilistic
verdict to veto anything. Kept import-free of pi APIs so `node --test`
exercises the policy directly.

## Modes (`/uma immune off|warn|ask|auto [--global]`)

- **off** — the interceptor does not run.
- **warn** (default) — warnings as notifications, never disturbing the flow.
- **ask** — each warning set is a confirm dialog; **a decline blocks the
  tool call with the warning as the reason**. The AI proposes, the operator
  disposes — consented blocking, the consent model intact.
- **auto** — today behaves like `ask` with an explanatory line: the
  recorded decision reserves silent auto-blocking for *deterministic
  contract-backed rules* (proposal 03). When contracts land, auto blocks
  contract violations outright and still asks for heuristic warnings.

## Invariant

**A heuristic verdict may notify or ask; it may never silently veto.** The
only path to a block without a dialog is a deterministic contract (not yet
implemented — SPAI-003). Warnings never reach the model transcript — UI
only. Thresholds are live-fixture-validated (the VSA probe warns, benign
edits stay silent) and pinned by tests.
