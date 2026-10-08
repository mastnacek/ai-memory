---
id: 01M4DDW00TZ76W7S4TYKP6DXRC
scope: "project:ai-memory"
type: pattern
title: "Verify rebuildable-cache schema via PRAGMA, not stored metadata"
description: Detect an outdated SQLite FTS index by inspecting real columns
tags:
  - sqlite
  - fts5
  - cache
  - schema
  - migration
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:35:19.066603700+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:35:19.066605200+00:00"
since: "2026-10-08T09:35:19.066605200+00:00"
---
### Context
The centralized FTS5 index stores a schema version in an `uma_meta` table. An earlier build stamped the new version without actually recreating the table, so trusting the recorded version left an old column layout in place and inserts failed with "no column named description".

### Pattern
For any rebuildable cache, treat the recorded version as a hint only and verify the *actual* structure before use. Here that means reading the real columns:
`PRAGMA table_info(facts_fts)` and requiring the expected column (e.g. `description`).
If it is missing, DROP the table and let the caller reindex from the Markdown source of truth.

### Consequences
- Self-healing after a broken upgrade; no manual `rm index.db` step.
- Safe because the data is derivable, so a drop loses nothing permanent.
- Do NOT apply this to non-derivable data — there a mismatch must refuse, not drop.