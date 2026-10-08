---
id: 01M4EFEF636KF3NDJSC2JXJ35Z
scope: "project:mozek_rust"
type: decision
title: "Mozek adds Google OAuth sign-in, mirroring the Pi plugin's approach"
tags:
  - mozek-rust
  - auth
  - google-oauth
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T19:22:07.427352600+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T19:22:07.427354400+00:00"
since: "2026-09-14T23:59:59+00:00"
---
## Context

Imported from session 01a09f1b (2026-09-14, project deleted from disk). The operator compared Mozek's backend with the working Pi plugin integration.

## Rule

Mozek gains Google OAuth sign-in, implemented the same way the Pi plugin already does it — one auth approach, reused, not reinvented.

## Consequences

- The Pi plugin's OAuth flow is the reference implementation.
- Any second auth provider needs a new decision.