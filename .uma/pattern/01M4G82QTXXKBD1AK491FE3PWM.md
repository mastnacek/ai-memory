---
id: 01M4G82QTXXKBD1AK491FE3PWM
scope: "project:ai-memory"
type: pattern
title: Two-skill layout lives inside the global UMA Pi package
tags:
  - skills
  - pi
  - layout
  - discovery
status: stable
supersedes: 01M4DEY3Z19C4NHYX41AWF4WPT
generated:
  by: pi-agent/1.1
  at: "2026-10-09T11:51:51.901443+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-09T11:51:51.901443900+00:00"
since: "2026-10-08T09:53:57.217427300+00:00"
---
### Context
The two-skill layout (general vs Pi-specific) is unchanged in spirit —
the skills split by audience with distinct names. What changed is the
physical location: the UMA Pi extension is now installed as a GLOBAL
package from the folder `uma-pi-extension/` (pi install, settings.json),
and pi packages discover skills from the conventional `skills/`
directory inside the package. Project-local `.pi/skills/uma-memory-pi`
was moved into the package, so the skills load in every project.

### Rule
- Pi-specific skill: `uma-pi-extension/skills/uma-memory-pi/SKILL.md`
  (canonical, loaded globally with the package).
- General skill: canonical at repo root `skills/uma-memory/SKILL.md`
  (harness-agnostic source of truth, referenced by AGENTS.md); the
  package ships a synced mirror at
  `uma-pi-extension/skills/uma-memory/SKILL.md` annotated as such, so
  globally installed Pi agents can read it outside this repo.
- Keep the two audiences distinct (do not merge); keep the mirror in
  sync when the canonical general skill changes.

### Consequences
- Skill availability no longer depends on the project.
- In this repo, the skills come from the global package (no project
  `.pi/skills` duplicates — double-named skills would collide).