# Translate Slice

## What

Display-only Czech translation of a proposal's body for the approval
modal. `translateForDisplay` calls the session's current model via
`completeSimple`; `buildTranslateMessages` is the pure, tested prompt
contract (structure, code, ULIDs and tags preserved verbatim; prose only).

## Why

The operator writes and reviews facts in English, but reads faster in
Czech. The modal should present the proposal in the operator's language —
while what UMA stores must stay byte-identical to what was proposed.

## Invariant

**Translation never touches stored data.** The result is a separate
display cache; the proposal object that the modal returns on approval
carries the original body. If a translated body ever flows into
`ProposalResult`, this slice's reason for existence is broken. Failure
degrades to showing the original (translation is a convenience, not a
transformation), and this slice never writes to memory or disk.