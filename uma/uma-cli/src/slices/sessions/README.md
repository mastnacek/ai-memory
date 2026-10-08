# `sessions` slice

**Command:** `uma sessions <list|show>`
**Depends on:** `uma-core::sources` (read-only session-store reader)

## What it does

A read-only browser over the **agent session stores** — pi agent and Claude Code — so a project's history can be inspected even after the project itself is deleted from disk.

- `sessions list [--project <text>] [--source pi|claude|all] [--sort new|old|size] [--limit N] [--json]` — one line per session: source, date, a `d` marker when the project directory no longer exists, user/substantive turn counts, size, project and title.
- `sessions show <id-or-path> [--json]` — one session in full: header, turn counts, and its **substantive user messages**.

## Why it exists

Sessions live in the *profile*, not the projects, so they outlive deletion — reconnaissance found 578 sessions across 4.5 months, **122 of them from projects deleted from disk** (`mozek_rust` included). Those sessions are the only surviving record of that work, and until now there was no way to even list them. This slice is also the browsing half of the import workflow (`uma import sessions`, future), and the separation is deliberate: **browsing reads, only the approval modal writes.**

## Invariant

- **Read-only.** It opens session files with `BufReader` and streams them line by line; it never writes, indexes, or proposes anything.
- **Provenance comes from the session's own `cwd`, never the directory name.** The encoded directory name is lossy and machine-specific, and the same project appears under several names (Windows + its WSL twin).
- **Secrets are masked, never reprinted.** Old sessions can contain real API keys; a browser must be safe to scroll through.
- **A broken session is listed, not dropped.** Unparsable lines are skipped; a file whose header is readable still yields a record, because partial history beats silent loss.
