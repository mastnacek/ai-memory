# risk — the file pain score

## Command

`uma risk pain <PATH> [--max-commits N] [--json]`

Scores a file 0–100 from its history of hurt: memory corrections mentioning
it (+15 each, cap 45), git reverts (+20 each, cap 60), and churn inside the
observation window (+2 per commit, cap 20). Three bands: low (0–20, normal),
medium (21–60, run the affected tests after editing), critical (61–100,
test-first — propose the failing test before the fix).

## Why it exists

The deterministic half of `docs/proposals/04` (Somatic markers / Pain
Score): humans flinch at files that burned them before. Memory already knows
which files hurt (corrections); git knows what was undone. Putting both into
one advisory score lets the interceptor warn *before* an edit lands, without
trusting a probabilistic model for anything.

## Invariant

**Advisory, deterministic, read-only.** The score never blocks and never
mutates: gathering (memory + git) lives here, scoring is pure kernel math
reproducible in `cargo test`, and caps keep any single signal from owning
the verdict (churn alone can never leave the low band — a busy file that
never broke is not dangerous). Probabilistic refinement may layer on top;
the floor must not depend on a network model.