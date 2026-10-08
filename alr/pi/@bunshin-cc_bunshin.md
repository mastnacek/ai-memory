# Bunshin

Bunshin lets your company’s AI agents (and robots) learn from each other. Your repo’s own SPEC.md, TESTING.md and SKILL.md sections, other teams’ rules and what earlier sessions learned reach the agent when they matter; what a session learns is tried locally, then reviewed and shared.

## Connect a project

You need Node.js 20 or later, a signed-in Codex, Claude Code, or Pi installation, and an account in your company’s [Bunshin workspace](https://bunshin.cc/dashboard/).

```sh
npm install -g @bunshin-cc/bunshin
cd /path/to/your/project
bunshin setup --harness codex
```

In Bunshin, open **Agent keys** and create a key for this computer. Paste it into the hidden setup prompt. Setup reads your role, team, and permissions from the workspace, saves the key in a private file outside the project, and configures the project hooks (five for Codex, six plus an MCP server for Claude Code). It does not need a separate model API key.

Run `codex` interactively in the same project. Review and accept its project trust prompt, then run `/hooks` and review and trust the five Bunshin hooks. Project trust and hook trust are separate steps. If `/hooks` does not list Bunshin, check that you opened the connected project and trusted its configuration. This guide tests interactive sessions; `codex exec` is not a substitute for this setup check.

For a linked Git worktree, setup puts Codex hooks in the primary checkout, where Codex reads them. The printed path identifies that shared hook file. Each connected worktree keeps its own `.bunshin` store and private connection. Worktrees created from a bare repository are not supported yet; use a full checkout.

For Claude Code, use `bunshin setup --harness claude-code`, then start a new session in the project and approve the `bunshin` MCP server it adds to `.mcp.json` (it gives the model the `bunshin_recall` and `bunshin_load` tools). `/learn` in the Claude Code plugin, or `bunshin learn "<what to remember>"`, captures knowledge on demand.

To connect a whole folder of projects (each its own repository), run setup there with `--hooks user`: the hooks go in your user settings and every session under that folder uses its workspace, while sessions elsewhere are untouched. `bunshin sources` shows and changes which of its folders take part (`exclude <folder>`, `only <folders…>`, `include <folder>`, `all`).

For Pi, use Node.js 22.19 or later and run `bunshin setup --harness pi`. It installs a project extension in `.pi/extensions/bunshin.js`. Start Pi in the project (or run `/reload` in an existing session), review its project resources, and check `/bunshin status`. The extension ships in this npm package; you do not need a second package or a repository checkout.

## Check it works

```sh
bunshin status
bunshin sync --json
bunshin index
bunshin recall "changing a state transition other teams depend on" --hypothetical "the owning team must approve first"
```

`bunshin index` shows how the repo’s docs split into sections and which paths push each one. Learned knowledge arrives after real sessions: when a session was worth it, a reflection runs at its end (for Claude Code, as a fork of the session, so it reuses the prompt cache). Suggestions that were not applied automatically wait in `bunshin review`.

## Staying current

The hooks run whichever `bunshin` is installed; nothing updates it for you. At most once a day, session start checks npm for a newer version and, when there is one, tells you once that day (`npm i -g @bunshin-cc/bunshin@latest`). Set `BUNSHIN_NO_UPDATE_CHECK=1` to turn the check off.

Sessions uploaded by a CLI before 0.6.2 carry a clipped trace (at most 60,000 characters). After updating, `bunshin sync --retrace` re-sends those from the last 30 days (`--days N`) with their whole trace, as long as the session's transcript file is still on the machine.

`bunshin demo` runs an offline example using temporary synthetic data. It does not verify your workspace connection or automatic learning.

## Scope and removal

Setup applies to the current project. The local `.bunshin/` directory contains lessons and session state and is added to `.gitignore`. The key is stored under `~/.config/bunshin/credentials/` with owner-only permissions. Your existing hooks remain in place.

```sh
bunshin init --harness codex --remove-hooks
npm uninstall -g @bunshin-cc/bunshin
```

Use `--harness claude-code` to remove Claude Code hooks or `--harness pi` to remove the Pi extension. Revoke this computer’s key in Bunshin when disconnecting it. Removing hooks keeps local lessons.

Codex hooks are shared across a repository's linked worktrees. Removing them from one removes the repository's Bunshin hooks for all its worktrees. Other tools' hooks are preserved.

Full setup guide: [bunshin.cc/docs](https://bunshin.cc/docs/).

