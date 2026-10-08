# fastbrain — the recall gate hook (S3 compromise)

## Behavior

A `before_agent_start` hook: when the operator's `recallGate` toggle is on,
each user prompt is checked by the judge (`uma recall check`, offline markers
or Jev via the `fastbrainJudge` config). A trigger recalls the top facts as
one hidden custom message (`uma-recall`, cache-stable, not a system-prompt
edit). No trigger, no toggle, no binary, or a timeout — the run proceeds
with no recall, always within the 8s gate bound.

## Why it exists

S3 (auto-injection) was paused by operator preference: uncontrolled injection
caused context noise and hallucination risk. This is the recorded compromise
(PRD backlog): injection becomes *conditional* — the fast judge opens memory
only for prompts that may depend on remembered decisions, and recall itself
stays offline BM25 so a turn never pays network cost for memory.

## Invariant

**Advisory only, and fail-quiet by design.** The gate can delay a run by at
most the timeout and can never fail one; every failure mode collapses to
"no recall". It injects nothing when the judge says no, and the hook's
subscription is tracked and drained like every other pi.on() in this
extension (the invariant that cost this file a rewrite). The operator's
toggle — default OFF — is the single source of truth; nothing here overrides
the paused-S3 decision on its own.
