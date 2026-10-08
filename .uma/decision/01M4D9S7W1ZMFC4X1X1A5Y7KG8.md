---
id: 01M4D9S7W1ZMFC4X1X1A5Y7KG8
scope: "project:ai-memory"
type: decision
title: Centralized Single SQLite FTS5 Index in User Profile
since: "2026-10-08T08:23:54.497882400+00:00"
tags:
  - sqlite
  - fts5
  - indexing
  - architecture
  - storage
---
### Context
Initially, each project repository stored its own local SQLite `index.db` inside `.uma/`. This caused repository clutter (binary database in git worktrees), required complex cross-database merging logic, and made cross-project knowledge retrieval inefficient.

### Decision
The project adopts a single, centralized SQLite FTS5 database stored exclusively in the user profile (`~/.local/share/uma/index.db` on Linux/macOS, `%APPDATA%/uma/index.db` on Windows).
- **Project repositories**: Contain only transparent, readable `.md` Markdown files in `.uma/<type>/<id>.md` with zero binary clutter.
- **Global space**: Stores global user preferences in `~/.local/share/uma/global/<type>/<id>.md`.
- **Single Central Index**: All facts across all projects and global memory are indexed in the single user-profile SQLite FTS5 database.

### Consequences
- Mathematically accurate global BM25 ranking across all facts.
- Repositories remain 100% clean of binary database files.
- Instant cross-project search capabilities without multi-DB connection overhead.