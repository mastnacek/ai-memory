---
id: 01M4DGRW9Q85PXXE11G9NA0BWQ
scope: "project:ai-memory"
type: decision
title: "Skill templates are expanded, never executed"
tags:
  - skill
  - s6
  - safety
  - execution
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T10:26:02.679870200+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T10:26:02.679874200+00:00"
since: "2026-10-08T10:26:02.679874300+00:00"
---
### Context
Procedural memory stores commands (slice S6). The PRD originally specified that `uma_skill_invoke` "expands template + runs".

### Decision
**Expansion only.** `uma_skill_invoke` / `uma skill invoke` return the resolved command as *text* — the JSON output carries `"executed": false`. The agent or human then runs it through their own shell tool, so the harness's command approval still applies.

Rejected alternatives:
- **Execute the rendered template** — turns a memory API into a code-execution surface and bypasses whatever approval the harness already applies to commands.
- **Per-skill opt-in plus binary allow-list** — still makes UMA an executor and doubles the validation surface for little gain.

### Consequences
- UMA never runs a command, so a memory store cannot be used as an execution primitive.
- Missing placeholders are reported and left **visible** in the output, never silently blanked into a command that would do something different.
- Expansion is pure and lives in `uma-core/src/skill.rs`, so it is unit-tested with no store and no shell.