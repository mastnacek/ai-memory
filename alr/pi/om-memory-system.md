# OMMS: Opinionated Modular Memory System

[![npm version](https://img.shields.io/npm/v/om-memory-system.svg)](https://www.npmjs.com/package/om-memory-system)
[![npm downloads](https://img.shields.io/npm/dm/om-memory-system.svg)](https://www.npmjs.com/package/om-memory-system)
[![license](https://img.shields.io/npm/l/om-memory-system.svg)](https://www.npmjs.com/package/om-memory-system)

![OMMS banner](.github/banner.png)

OMMS gives your AI coding agent a long-term memory. As you work, it writes
short notes about what was done and decided in each project: fixes, design
choices, things that did not work. It brings the relevant notes back in later
sessions, so the agent does not start from zero every time. It also learns how
you like to work and keeps that as a user profile.

It runs inside [OpenCode](https://opencode.ai) and the
[Pi coding agent](https://www.npmjs.com/package/@earendil-works/pi-coding-agent).
Both share one memory per project, so a note written in one is available in
the other. Everything is stored locally on your machine.

Claude Code is also supported, through a plugin of hooks. It needs Node.js and
your own external API for capture. A global install is optional. See the
[Claude Code adapter](docs/claude-code-adapter.md) guide.

## What it does

- **Remembers automatically.** After each piece of work, a background model
  call summarises it into a memory. You do not have to ask.
- **Recalls when relevant.** Matching memories are added to the agent's
  context: on every prompt in OpenCode v2 and Pi, at the start of a session in
  OpenCode v1.
- **Learns your preferences.** A user profile of your habits builds up over
  time and follows you across projects.
- **Imports your past sessions.** Pi and OpenCode import older history after
  startup. A command gives you a preview and manual control.
- **Lets you look and edit.** A local web page shows every memory and your
  profile.
- **Keeps private things private.** Text inside `<private>` tags is never
  stored.

![Project memory timeline](.github/screenshot-project-memory.png)

![User profile viewer](.github/screenshot-user-profile.png)

## Before you start

You need one of:

- **OpenCode** 1.18.29 or later (v1 plugin API) or OpenCode v2
- **Pi coding agent**
- **Claude Code**, through a plugin of hooks. Capture needs your own external API. See
  the [Claude Code adapter](docs/claude-code-adapter.md) guide for setup.

Nothing else is required. OMMS brings its own database. On first use it
downloads a small embedding model (the part that makes memories searchable),
so you need internet access once. The terminal import command also needs
Node.js 22.14 or later.

## Set up

### 1. Install

**OpenCode.** Add `om-memory-system` to `~/.config/opencode/opencode.json`
(on Windows, `%USERPROFILE%\.config\opencode\opencode.json`):

```jsonc
// OpenCode v2
{ "plugins": ["om-memory-system"] }

// OpenCode v1
{ "plugin": ["om-memory-system"] }
```

On OpenCode v2 you can run `opencode plugin add om-memory-system` instead.
Restart OpenCode.

**Pi.** Run:

```bash
pi install npm:om-memory-system
```

Restart Pi. You can install OMMS in both agents; they share the same memory.

**Terminal command (optional).** A global install puts the `om-memory-system`
command on your `PATH`. You do not need it. OMMS runs the newest copy it finds on
the machine, so updating one host is enough. A global install from 4.3 or earlier
needs one update. See
[Updating and upgrading](docs/upgrading.md#one-update-updates-every-host).
To install it:

```bash
npm i -g om-memory-system      # or: bun add -g om-memory-system
om-memory-system --version
```

### 2. Choose which model writes memories (optional)

With no settings, OMMS uses the model of the session you are working in. To
use a different one, add it to `~/.config/omms/omms.jsonc`. If you have no
config file yet, OMMS creates this one with comments on first start.

```jsonc
{
  // A model you are already signed in to in OpenCode or Pi.
  // Use "inherit" to always follow the session's model.
  "opencodeProvider": "openai",
  "opencodeModel": "gpt-5.6-luna",
  "piProvider": "openai-codex",
  "piModel": "gpt-5.6-luna",
}
```

You can also use any OpenAI-compatible or Anthropic API with your own key, set
up on the Settings page's **External API** card, and choose `"external"` as a
host's model. See [Configuration](docs/configuration.md#choosing-the-model).

### 3. Check it works

Work normally for a few turns, then open `http://127.0.0.1:4747` in your
browser. OpenCode or the login web app serves this page. New memories appear
on the timeline. You can also ask the agent: "search memory for what we changed today".

## Import your past history

Older sessions import automatically after a host starts. This makes model
calls. To turn it off, set `"autoBackfill": false` in
`~/.config/omms/omms.jsonc` before you start the agent. For a manual import or
a custom source, preview it inside the agent:

```text
/memory-import-opencode-history --dry-run
/memory-import-pi-history --dry-run
```

The preview shows how many model calls a real import needs. It uses the
session's model; add `--model provider/id` for a cheaper one. There is also a
terminal version, `npx om-memory-system`, that uses an API key. See the
[OpenCode](docs/opencode-history-import.md) and [Pi](docs/pi-history-import.md)
import guides, and the [CLI reference](docs/cli.md).

## Keeping OMMS up to date

Pi shows a notice when a new version is out; update with
`pi update npm:om-memory-system`. On OpenCode v2, the footer shows when a new version is out;
run `opencode plugin update om-memory-system`. Restart the agent afterwards.
[UPDATES.md](UPDATES.md) explains how updates work on every host.
To stay on one version, install it with the number, for example
`pi install npm:om-memory-system@3.1.1` or
`opencode plugin add om-memory-system@3.1.1`; a pinned install is never
updated. Coming from `opencode-mem`? Your memories move over automatically, with a
backup first. See [Updating and upgrading](docs/upgrading.md) and
[CHANGELOG.md](CHANGELOG.md).

## Documentation

| Read this                                                        | To learn about                                                   |
| ---------------------------------------------------------------- | ---------------------------------------------------------------- |
| [Using memory day to day](docs/using-memory.md)                  | How capture and recall work, the `memory` tool, the user profile |
| [Configuration](docs/configuration.md)                           | Settings, choosing the model, embeddings, troubleshooting        |
| [Web UI](docs/web-ui.md)                                         | The memory explorer, opening it on a network safely              |
| [Settings page](docs/web-ui-settings.md)                         | Every card and control on the web Settings page                  |
| [Moving projects](docs/moving-projects.md)                       | Nested repositories, moved folders, backup and restore           |
| [Updating and upgrading](docs/upgrading.md)                      | Updates, pinning a version, older stores                         |
| [OpenCode adapter](docs/opencode-adapter.md)                     | How the OpenCode plugin hooks in                                 |
| [Pi adapter](docs/pi-adapter.md)                                 | How the Pi extension hooks in                                    |
| [Claude Code adapter](docs/claude-code-adapter.md)               | Installing the Claude Code plugin, hooks, the `memory` command   |
| [OpenCode history import](docs/opencode-history-import.md)       | Importing past OpenCode sessions                                 |
| [Pi history import](docs/pi-history-import.md)                   | Importing past Pi sessions, moving machines                      |
| [Claude Code history import](docs/claude-code-history-import.md) | Importing past Claude Code transcripts                           |
| [CLI reference](docs/cli.md)                                     | The `om-memory-system` terminal command                          |
| [Migrating from opencode-mem](docs/omms-migration.md)            | What changes when upgrading from the original plugin             |
| [For developers](docs/developers.md)                             | Building, testing, the public `tags` export, architecture        |
| [Contributing](CONTRIBUTING.md)                                  | How to set up, test, and send a pull request                     |

## About this fork

OMMS is [`cmdaltctr/omms`](https://github.com/cmdaltctr/omms), a fork of
[`tickernelz/opencode-mem`](https://github.com/tickernelz/opencode-mem),
published on npm as `om-memory-system`. The fork runs the memory engine as a
shared core for both OpenCode and Pi (see [shared core](docs/shared-core.md)).
Existing OpenCode memories and settings keep working.

## License and links

MIT License. See [LICENSE.md](LICENSE.md).

- Repository: https://github.com/cmdaltctr/omms
- Upstream: https://github.com/tickernelz/opencode-mem
- Issues: https://github.com/cmdaltctr/omms/issues

Inspired by [opencode-supermemory](https://github.com/supermemoryai/opencode-supermemory).

