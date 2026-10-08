---
id: 01M4DDYZNJ7FPWQDYSDSHH47DH
scope: "project:ai-memory"
type: pattern
title: Reindex merges known roots instead of wiping them
description: "Preserve roots recovered from indexed file paths so one project's reindex keeps others"
tags:
  - reindex
  - index
  - robustness
  - pattern
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:36:57.010436300+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:36:57.010437700+00:00"
since: "2026-10-08T09:36:57.010437800+00:00"
---
### Context
`reindex_from_dirs` began with `DELETE FROM facts_fts` and then scanned only the roots it was handed (global + current project). Reindexing from one repository silently removed every other project's rows from the shared index.

### Pattern
Before wiping a rebuildable cache, recover the full set of roots already known to it. Here that means reading `SELECT DISTINCT file_path` and deriving each store root as the grandparent of `<root>/<type>/<id>.md`, then scanning those roots too. Roots that no longer exist are skipped, which is exactly the orphan-pruning behaviour we want.

### Consequences
- Cross-project search survives a reindex run from any single project.
- Missing/deleted roots are pruned as a side effect, with no separate cleanup pass.
- Embeddings are pruned to the rebuilt fact set in the same step.