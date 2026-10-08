# recall — the fastbrain gate over memory recall (S3)

## Command

`uma recall check <PROMPT> [--judge off|jev] [--max N] [--json]`

Decides whether a prompt may depend on remembered facts (offline markers, or
Jev via OpenRouter with `--judge jev`), and only on a trigger runs an offline
BM25 keyword search, returning the hits with IDs, types, and snippets.

## Why it exists

Auto-injection (S3) was paused deliberately — uncontrolled injection caused
context noise. The fastbrain gate makes injection *conditional*: casual
messages recall nothing, messages that reference past decisions recall a
handful of facts. This is the recorded compromise in the PRD backlog.

## Invariant

**Read-only, and recall is offline.** The gate decides *whether to search*;
the search itself is always the local BM25 index — never a network semantic
call, so a turn never pays latency or API cost for recall. When the Jev
judge is unavailable the verdict degrades to the offline marker judge *with
a visible note*, never silently. The caller decides what to do with the
hits — UMA injects nothing on its own.
