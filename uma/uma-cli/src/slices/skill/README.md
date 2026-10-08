# `skill` slice

**Command:** `uma skill <new|list|show|invoke>`
**Depends on:** `uma-core::{skill, domain, store}`

## What it does
Stores *procedural* memory. A skill is a `skill` fact whose `template` frontmatter field holds a command with `{{placeholder}}` slots.
- `skill new --name docker-build --template "docker build -t {{tag}} ."` creates one.
- `skill list` shows every skill and the placeholders it needs.
- `skill show <name|id>` prints the fact, template included.
- `skill invoke <name|id> --set tag=v1` **expands** the template and prints the result.

## Why it exists
Facts capture *what is true*; skills capture *how to do something*. Re-deriving a working command every session is exactly the repeated work durable memory should eliminate.

## Invariant
- **UMA never executes a template.** `invoke` is pure string substitution whose only output is text (JSON carries `"executed": false`). Running the command is the caller's decision, so the caller's own approval applies — a harness's shell tool, or a human at a terminal. A memory store that can run shell commands is a code-execution surface, and that is out of scope by design.
- Missing placeholders are reported and left **visible** in the output, never silently blanked into a command that would do something different.
- Template expansion is pure and lives in `uma-core/src/skill.rs`, so it is unit-tested without a store or a shell.
