---
id: 01M4DQ9C37YR0J7ZG358DQQA6F
scope: "project:ai-memory"
type: decision
title: Sync is git-based over the global store only
tags:
  - sync
  - s8
  - git
  - design
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T12:19:54.599912400+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T12:19:54.599915700+00:00"
since: "2026-10-08T12:19:54.599915700+00:00"
---
### Context
With sync (S8) the question was how memory moves between machines. Facts are already OKF Markdown files, so the two candidates were git and a custom rsync/bundle protocol.

### Decision
**Git-based, and the global store only.** `uma sync push|pull|status` operates on a repository at the global store root:
- **Global store only, deliberately.** Project memory lives inside the project's own repository and travels with it when `.uma/` is tracked. Sync must never commit *or push* an operator's work branch — a memory tool that pushes someone's branch is a surprise waiting to happen.
- **rsync was rejected** because the merge policy for prose conflicts is the hard part, and git already implements it.
- **Conflicts are the operator's job.** A conflicting merge stops, reports unmerged files and any leftover `<<<<<<<`/`>>>>>>>` markers, and never picks a side: a conflict inside YAML frontmatter makes a fact unparsable, and the index would skip it silently.
- **The cache is never published.** `index.db` sits *inside* the global root and changes on every search; `.gitignore` is written on every push and a repo that already tracks it is untracked automatically.
- Sync is CLI/operator-facing only — no Pi tool, no MCP tool. Agents never trigger pushes.

### Consequences
- The first push on a machine without a remote still creates history, so adding a remote later publishes everything in one step.
- Fact counts in commit messages exclude housekeeping, so "N fact(s) changed" stays truthful.
- Found live during testing: the very first real push committed `index.db` (the ignore had been planned but not implemented); the healing push untracked it and every push now guards it.
- Operations take the repo root explicitly, so tests run against throwaway directories and can never touch live memory.