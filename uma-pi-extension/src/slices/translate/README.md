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

## Model resolution (option A — shared knob with the operator's plugin)

`resolveTranslateModel` reads the translate plugin's global config
(`~/.pi/agent/pi-prompt-translate.json` → `translateModel`) and resolves:
explicit `provider/id` → pi's default model (`settings.json`) → the
session's current model. Every failure degrades to the session model —
translation keeps working, only the bill changes. Pure parsing lives in
`model.ts` (import-free of pi-ai values, unit-tested); `prompt.ts` is the
import-free prompt contract. Both must stay value-import-clean so
`node --test` can load them.

## Fact views (read / search / list)

`translateOutputForDisplay` translates a whole command output block with
ONE model call per command, under the same display-only invariant: the
CLI output, memory files and index are untouched. Consumers reach this
slice only through `shared/translate_client.ts` (no cross-slice imports).
A failed translation degrades to the original text with the reason
visible in the output note.

## Invariant

**Translation never touches stored data.** The result is a separate
display cache; the proposal object that the modal returns on approval
carries the original body. If a translated body ever flows into
`ProposalResult`, this slice's reason for existence is broken. Failure
degrades to showing the original (translation is a convenience, not a
transformation), and this slice never writes to memory or disk.