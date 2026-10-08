# `search` slice

**Command:** `uma search "<query>" [--mode keyword|semantic|hybrid] [--scope] [--type] [--as-of] [--include-deprecated] [--limit] [--reindex] [--vectorize]`
**Depends on:** `uma-core::{search, indexer, embeddings, vector_store}`

## What it does
Three retrieval modes over one fact set:
- `keyword` — BM25 via SQLite FTS5, with weighted `snippet` highlighting.
- `semantic` — cosine similarity over stored embeddings (OpenRouter).
- `hybrid` (default) — Reciprocal Rank Fusion (k=60) of both.

Supports point-in-time queries (`--as-of`) and hides deprecated facts unless asked.

## Why it exists
This is the on-demand recall path. Automatic context injection (slice S3) is deliberately on hold because injecting facts every turn buys noise and hallucination; an explicit query keeps the agent in control of what enters its context.

A `stale_after` date is honoured the same way as `until`: past it, the fact leaves the default result set. That is the point of a validity deadline — an expired claim should not look current just because nothing replaced it.

Two behaviours worth knowing when reading results:

- **Visible degradation.** `hybrid` needs an embedding client, stored vectors, and a working API call. When any is missing it degrades to keyword-only and prints a note *before the results* (`! Hybrid search degraded to Keyword: …`), because a caller who asked for hybrid ranking and silently received BM25 has been misled, not merely served differently. `--mode semantic` fails loudly instead; `--mode keyword` never needs a key.
- **Scope isolation.** Unscoped search sees only the current project plus global. Use `uma scopes` to discover what other scopes exist, then `--scope <name>` to reach them.

## Invariant
- The index is a rebuildable cache, never the source of truth — `--reindex` must always reconstruct it from the Markdown files.
- Index writes are gated on store canonicality, so temp/test stores cannot pollute the shared database.
