---
id: 01M4EF3BYREPCY41FN5AMTYG8K
scope: "project:ai-memory"
type: decision
title: "Mozek uses embeddings only for News-tab summarization, never for search"
tags:
  - mozek-rust
  - embeddings
  - architecture
status: deprecated
generated:
  by: pi-agent/1.1
  at: "2026-10-08T19:16:03.672539400+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T19:16:03.672541200+00:00"
since: "2026-10-08T19:16:03.672541200+00:00"
until: "2026-10-08T19:18:31.149781400+00:00"
---
## Context

During the Antigravity/Gemini integration work on the Mozek application (session imported from 2026-09-14), the operator set the role of embeddings in the app.

## Rule

Embeddings serve **only summarization in the News tab**. They are not a search mechanism and must not be added to search or other features without a new decision.

## Consequences

- No vector-search infrastructure belongs in Mozek's backend.
- Feature requests that imply embedding-based search need explicit operator approval first.