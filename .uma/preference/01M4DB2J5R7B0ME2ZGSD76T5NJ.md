---
id: 01M4DB2J5R7B0ME2ZGSD76T5NJ
scope: "project:ai-memory"
type: preference
title: Require manual reload before testing modified Pi plugins
since: "2026-10-08T08:46:28.536386+00:00"
tags:
  - preference
  - pi-plugin
  - reload
  - workflow
---
### Context
When developing or modifying Pi extensions and plugins, executing tool functions immediately before the operator has reloaded the session causes runtime mismatch and disrupted interaction.

### Preference / Rule
- Never autonomously execute newly created or modified Pi plugin functions or tools without the operator having performed `/reload` and explicitly confirmed readiness to test.
- Always wait for explicit user confirmation after touching plugin extension code.

### Consequences
- Prevents UI crashes and out-of-sync session states.
- Ensures all TypeScript and TUI component changes are cleanly loaded in the runtime before invocation.