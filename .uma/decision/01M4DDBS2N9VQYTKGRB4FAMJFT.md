---
id: 01M4DDBS2N9VQYTKGRB4FAMJFT
scope: "project:ai-memory"
type: decision
title: Adopt OKF v0.2 as the UMA frontmatter standard
description: "UMA facts are valid Open Knowledge Format v0.2 documents, extended with id/scope/supersedes"
tags:
  - okf
  - frontmatter
  - format
  - lifecycle
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:26:27.669154700+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:26:27.669156400+00:00"
since: "2026-10-08T09:26:27.669156400+00:00"
---
### Context
Google's Open Knowledge Format (OKF) v0.2 is an open, vendor-neutral standard for representing knowledge as Markdown + YAML frontmatter. UMA already used that shape, so aligning avoids inventing a bespoke format.

### Decision
UMA facts are valid OKF v0.2 concept documents, extended with UMA-specific keys:
- OKF keys: type, title, description, tags, status (stable|deprecated|draft), generated {by,at}, verified [{by,at}], since, until, stale_after.
- UMA extensions: id (ULID), scope (global | project:<name>), supersedes (ULID).
- Trust tier is derived from verified: human: actor => human-reviewed.

### Consequences
- Bundles are portable and readable by any OKF-aware tool; no lock-in.
- Supersession maps onto OKF lifecycle (status: deprecated + until) instead of a custom scheme.
- The migration slice (uma migrate) rewrites legacy files and backfills provenance.