# `export` slice

**Command:** `uma export [-s <scope>] [-t <type>] [--include-deprecated] [--okf --out <dir>]`
**Depends on:** `uma-core::serialization`

## What it does
Produces a portable copy of memory:
- **Default / `--json`** — a JSON array on stdout, one object per fact including its body and `template`.
- **`--okf --out <dir>`** — an OKF v0.2 bundle: one standalone Markdown document per fact at `<dir>/<type>/<id>.md`, plus a `MANIFEST.json` listing what was written.

Deprecated facts are excluded unless `--include-deprecated` is passed.

## Why it exists
Memory must never be locked into this tool. Because facts are already OKF Markdown, an export is a plain directory that any editor can read, git can diff, and a future tool can consume. It is also the honest answer to "how do I get my data out" — which is why it is read-only and never transforms the content.

## Invariant
- **Read-only** with respect to the store: it writes only into `--out`, and never indexes, mutates, or deletes.
- `--okf` without `--out` is an error, and `--out` without `--okf` is an error — an export that silently wrote nowhere (or wrote a stray directory when the user expected stdout) would be worse than refusing.
- Documents are byte-identical to the canonical serializer, so an export re-imports without drift.
