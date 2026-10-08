# Redskilled

Software and distribution for the [RedSkills marketplace](https://github.com/reddb-io/red-skills):
MCP servers, host daemon, worker runtime, tunnel, mobile and desktop interfaces,
IDE/terminal integrations, hooks, installers and all published packages.

The repositories are siblings with independent versions. Package names, command
names, host identities and existing state locations remain stable.

## Development

Requires Node 22+, pnpm 11.5.0 and the tooling declared by each app.

```sh
node scripts/prepare-skills.mjs
REDDB_SKIP_POSTINSTALL=1 pnpm install --frozen-lockfile
pnpm bundle
pnpm typecheck
```

`skills.lock.toon` pins the exact RedSkills commit. `plugins/` and root marketplace
manifests are generated, ignored inputs, composed with implementations in `runtime/`.
Edit skills in RedSkills; edit software here. To test a coordinated candidate:

```sh
node scripts/prepare-skills.mjs --source ../red-skills
node scripts/check-skills-boundary.mjs ../red-skills
```

The explicit local source is for development only. CI/release always uses the lock.
Run `pnpm skills:prepare` after changing the lock and before tests or packaging.

## Installation and publishing

`scripts/install-runtime.sh <exact-version>` installs the existing core and plugin
npm packages and checks command availability. Hosts require the commands on PATH;
hooks and MCP startup do not fetch packages. `red-skills-resource` exposes helpers
extracted from the content repository. `red-skills-hook` owns host hook behavior.

The existing publisher now belongs here and reads the same organization-level
`NPM_TOKEN` and `RELEASE_PAT` secrets as RedSkills. `REDSKILLED_PUBLISH_ENABLED`
remains off until signing and consumers have passed the migration checks. See
[migration/README.md](migration/README.md).

Mobile publishing has its own `REDSKILLED_MOBILE_PUBLISH_ENABLED` switch. It may
remain off while runtime and npm releases proceed without Android credentials.

## Diagnostic logs

With the runtime already installed and selected through mise,
`mise exec -- redskilled logs --path` prints the daemon diagnostic file location
without starting a daemon. `mise exec -- redskilled logs --open` and **Open log**
in the tray menu open that existing file with the desktop's file association.

- Linux: `$XDG_STATE_HOME/redskilled/logs/daemon.log`, or
  `~/.local/state/redskilled/logs/daemon.log` when XDG state is unset/relative.
- Windows: `%LOCALAPPDATA%\redskilled\logs\daemon.log`.
- macOS: `~/Library/Logs/redskilled/daemon.log`.

The serving daemon retains at most **five files of 10 MiB each**: `daemon.log`
and `.1` through `.4` (newest archive first). Rotation runs while the daemon is
active, not just during upgrades. Files are owner-only where the OS supports
Unix permissions. Lines have timestamps and severity; oversized lines are
omitted with a marker. Known credentials and payload fields are redacted before
writing. Logs remain local; no diagnostic upload is performed.

The web server, remote-link host and relay use the same policy independently in
`web.log`, `link.log` and `relay.log` in that directory. Pairing, invitations and
other short CLI commands are not captured; machine-readable stdout is unchanged.

stderr/journal remains available, including when file writes fail. On Linux,
`journalctl --user -u redskilled.service` also covers early loader failures and
native crashes that cannot run a JavaScript handler. Older releases have no
diagnostic file until an upgraded daemon has started.

Structured state such as `~/.red/redskilled/redskilled.log.toonl`, death records,
and Worker logs keeps its existing format, paths and retention rules. Those
files are not interchangeable with the disposable diagnostic log.

## Plugin content

- [Dev](https://github.com/reddb-io/red-skills/tree/main/plugins/dev/)
- [Memory](https://github.com/reddb-io/red-skills/tree/main/plugins/memory/)
- [Brain](https://github.com/reddb-io/red-skills/tree/main/plugins/brain/)
- [Internal](https://github.com/reddb-io/red-skills/tree/main/plugins/internal/)

Example policy, not a default: create manual worktrees with
`git worktree add .red/tmp/worktrees/manual/<slug> -b <branch> origin/main`.
Repository command guards remain configured by the consuming project.

For skill routing use [`ask-red`](https://github.com/reddb-io/red-skills/blob/main/plugins/dev/skills/engineering/ask-red/SKILL.md).
Other maintainer interfaces: [`what`](https://github.com/reddb-io/red-skills/blob/main/plugins/dev/skills/productivity/what/SKILL.md),
[`wizard`](https://github.com/reddb-io/red-skills/blob/main/plugins/dev/skills/engineering/wizard/SKILL.md),
[`to-questionnaire`](https://github.com/reddb-io/red-skills/blob/main/plugins/dev/skills/productivity/to-questionnaire/SKILL.md),
and [`writing-for-agents`](https://github.com/reddb-io/red-skills/blob/main/plugins/dev/skills/productivity/writing-for-agents/SKILL.md).

[`guard-process-birth`](https://github.com/reddb-io/red-skills/blob/main/plugins/dev/skills/engineering/guard-process-birth/SKILL.md)

[`guard-serialization`](https://github.com/reddb-io/red-skills/blob/main/plugins/dev/skills/engineering/guard-serialization/SKILL.md)

[`redskilled`](https://github.com/reddb-io/red-skills/blob/main/plugins/dev/skills/engineering/redskilled/SKILL.md)

Personal facts belong in Brain, not Memory. Memory is not the Personal-fact store.
