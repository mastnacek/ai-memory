# `sync` slice

**Command:** `uma sync <push|pull|status>`
**Depends on:** `uma-core::store`, `git` (the binary, via `sync/git.rs`)

## What it does

Carries the **global** memory store between machines using git.

- `uma sync push` — creates the repository at the global store root if needed, stages every fact file, commits (with a generated `memory sync: N fact(s) changed…` message unless `--message` is given), and pushes. The first push sets the upstream, which is what later pulls use.
- `uma sync pull` — fetches, then merges the upstream branch. A fresh machine with an unborn branch takes the remote branch wholesale (`Restored`). After any successful pull the search index is rebuilt, because files changed underneath it.
- `uma sync status` — branch, remote, upstream, ahead/behind, working-tree state, last commit.

## Why it exists

Memory is only useful if it survives the machine it was written on. Because facts are already OKF Markdown, git supplies history, diffs and visible conflicts for free — there is no bespoke transport to build and no merge policy to invent. rsync-style bundle sync was rejected for exactly that reason: the merge policy for prose conflicts *is* the hard part, and git already implements it.

## Invariant

- **The global store only.** Project memory lives inside that project's own repository and travels with it when `.uma/` is tracked. This slice must never commit *or push* an operator's work repository — a memory tool that pushes someone's branch is a surprise waiting to happen. (Here `.uma/` is tracked, so project memory already moves with `git push`.)
- **Conflicts are the operator's job.** A merge that conflicts stops and reports the unmerged files; local content is preserved inside them. It never picks a side, because a conflict in YAML frontmatter makes a fact unparsable — and the index would then silently skip it, which is why every pull scans for leftover `<<<<<<<` / `>>>>>>>` markers and reports them loudly.
- **The cache is never published.** The repository root is the global store root, so the committed content is exactly the Markdown facts; `index.db` is a sibling in the user profile and must stay out of the repository.
- **Reported, not printed.** Push/pull/status return report values, so the logic is testable against a throwaway directory and can never touch live memory from a test.
