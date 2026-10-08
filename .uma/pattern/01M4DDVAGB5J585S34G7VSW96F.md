---
id: 01M4DDVAGB5J585S34G7VSW96F
scope: "project:ai-memory"
type: pattern
title: Pi discovers repo skills via .pi/settings.json paths
description: Point Pi at the repo skills/ directory instead of duplicating a skill under .pi/skills/
tags:
  - pi
  - skills
  - settings
  - plugin
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:34:57.035902300+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:34:57.035903900+00:00"
since: "2026-10-08T09:34:57.035904+00:00"
---
### Context
A skill placed only in `skills/uma-memory/` is invisible to Pi, whose default discovery is `.pi/skills/`. Copying it into `.pi/skills/` creates a second file that silently drifts from the original.

### Pattern
Declare the repo skills directory in project settings instead of duplicating:
`.pi/settings.json` -> `{ "skills": ["../skills"] }`
Project-settings resource paths resolve relative to the project `.pi` directory, so `../skills` points at the repo's `skills/` folder. Pi discovers it (verified: a non-interactive Pi run lists `uma-memory`).

### Consequences
- Exactly one copy of the skill; no drift.
- The same file also serves other harnesses that read `skills/<name>/SKILL.md`.
- Project skills require project trust, like extensions.