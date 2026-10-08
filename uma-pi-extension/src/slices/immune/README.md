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

## Invariant

**Advisory only.** Nothing in this slice can block a tool call, and its
warnings never reach the model transcript — UI notifications only. The
thresholds are live-fixture-validated (the VSA probe warns, benign edits
stay silent) and pinned by tests.
