---
id: 01M4EFG2AHFYNJFTGSFT5T1NAT
scope: "project:ai-memory"
type: pattern
title: "An approved proposal is the reviewed contract: unedited fields must survive the modal round-trip"
tags:
  - modal
  - contract
  - approval-gate
  - regression-lesson
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T19:22:59.793775700+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T19:22:59.793777400+00:00"
since: "2026-10-08T19:22:59.793777400+00:00"
---
## Context

The approval modal used to rebuild the approved proposal from only the editable fields, silently dropping supersedes/template/stale_after/since — masked for months by CLI-side inheritance, exposed by the first import that carried a 'since'.

## Rule

Whatever the tool proposed and the operator approved is the stored contract. The modal spreads the full initial proposal (edited fields on top); adding an optional field to a tool MUST include passing it through modal, args, and CLI/MCP counterparts.

## Consequences

- New proposal fields need round-trip coverage in all three surfaces (Pi tool, CLI flag, MCP param).
- A fact that stores wrong after approval is a modal/passthrough defect, not an authoring error.