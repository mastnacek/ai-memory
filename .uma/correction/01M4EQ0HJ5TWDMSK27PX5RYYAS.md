---
id: 01M4EQ0HJ5TWDMSK27PX5RYYAS
scope: "project:ai-memory"
type: correction
title: store_revision must never reset validity.since
tags:
  - store
  - validity
  - regression-lesson
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T21:34:19.717261200+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T21:34:19.717265200+00:00"
since: "2026-10-08T21:34:19.717265200+00:00"
---
## Context

Found while building the import flow: uma-core/src/store/ops.rs reset validity.since to now on every supersession (store_revision), silently moving every claim's origin forward and making the supersede --since flag a no-op at the storage layer.

## Rule

A supersession restates the claim: store_revision in uma-core/src/store/ops.rs must preserve the caller-provided validity.since; only an explicit caller override (imports restoring an origin date) may move it.

## Consequences

- Revisions inherit the predecessor's origin by default.
- The timeline shows revision times from generated.at, not since.