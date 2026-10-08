---
id: 01M4EFN11ZA3ZG026EAQ0XSEBZ
scope: "project:mozek_rust"
type: decision
title: "Mozek uses embeddings only for News-tab summarization, never for search"
description: Imported from dead-project session 01a09f1b; restores the original 2026-09-14 origin after the pre-fix supersede reset it.
tags:
  - mozek-rust
  - embeddings
  - architecture
status: deprecated
supersedes: 01M4EF7VZDSTMZVJBJ3VDTHRW9
generated:
  by: pi-agent/1.1
  at: "2026-10-08T19:25:42.335126400+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T19:25:42.335126600+00:00"
since: "2026-10-08T19:25:42.335689500+00:00"
until: "2026-10-08T19:34:32.177869600+00:00"
---
## Context

During the Antigravity/Gemini integration work on the Mozek application (session imported from 2026-09-14), the operator set the role of embeddings in the app.

## Rule

Embeddings serve **only summarization in the News tab**. They are not a search mechanism and must not be added to search or other features without a new decision.

## Consequences

- No vector-search infrastructure belongs in Mozek's backend.
- Feature requests that imply embedding-based search need explicit operator approval first.