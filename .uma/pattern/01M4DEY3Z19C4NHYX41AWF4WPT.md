---
id: 01M4DEY3Z19C4NHYX41AWF4WPT
scope: "project:ai-memory"
type: pattern
title: "Two-skill layout: general at skills/, Pi-specific at .pi/skills/"
description: "General skill in skills/, Pi-specific skill in .pi/skills/, distinct names"
tags:
  - skills
  - pi
  - layout
  - discovery
status: stable
supersedes: 01M4DDVAGB5J585S34G7VSW96F
generated:
  by: pi-agent/1.1
  at: "2026-10-08T09:53:57.217079300+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T09:53:57.217080200+00:00"
since: "2026-10-08T09:53:57.217427300+00:00"
---
### Context
An earlier pattern described a single skill at `skills/uma-memory/` loaded into Pi via `.pi/settings.json` (`"skills": ["../skills"]`), on the premise that duplicating the skill was the problem. That premise was wrong: the real requirement is two *different* documents for two audiences.

### Pattern
- `skills/uma-memory/SKILL.md` — the harness-agnostic skill. Pi loads it by declaring the repo directory in project settings: `.pi/settings.json` → `{ "skills": ["../skills"] }` (project-settings resource paths resolve relative to the project `.pi` directory).
- `.pi/skills/uma-memory-pi/SKILL.md` — the Pi-specific skill, discovered natively by Pi.

Both load into Pi under **distinct names**, so there is no collision. The Pi skill references the general file by path for shared concepts.

### Consequences
- Verified: a non-interactive Pi run reports both `uma-memory` and `uma-memory-pi`.
- The general skill stays correct for agents that have no Pi tools.
- Supporting a third harness means adding its own skill, not growing the general one.