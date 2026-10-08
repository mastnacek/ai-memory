# pi-dejavu-memory

> **Renamed:** former npm/GitHub package `pi-noc-memory` → **`pi-dejavu-memory`**. Prefer this package.


<p align="center">
  <img src="./assets/readme/hero.svg" width="100%" alt="pi-dejavu-memory — SessionStart boot protocol + memory tools for Pi">
</p>

**DejaVu extension for Pi — automated memory management with SessionStart boot protocol.**

Agent-side companion to [DejaVu](https://github.com/RealAlexandreAI/DejaVu) (the Cloudflare-hosted MCP memory server). Also available for dsh: [dsh-dejavu-memory](https://github.com/RetiredPhysicist/dsh-plugins/tree/main/packages/dsh-dejavu-memory).

## Features

- **SessionStart Boot Protocol** — automatically calls `noc_boot` (reads `system://boot`, `system://recent/5`, `system://triggers`, then best-effort `system://briefing`) at session start; then read `system://focus` to resume active working trees (`recent` is a briefing subset — no need to re-read it after boot)
- **Memory Rules** — global rules injected every session for intelligent memory usage (write-judgement, update-over-create, trigger discipline)
- **Memory Tools** — `noc_read`, `noc_create`, `noc_update`, `noc_delete`, `noc_search`, `noc_alias`, `noc_triggers`

## Install

```bash
pi install npm:pi-dejavu-memory
```

> **Upgrading from pi-nocturne-memory (≤1.0.x):** the package was renamed to `pi-dejavu-memory` and tools renamed from `nocturne_*` to `noc_*`. Old config at `~/.pi/agent/extensions/pi-nocturne-memory/config.json` is still read as a fallback, so your MCP URL/credentials keep working — just reinstall the new package and update any prompt text that referenced `nocturne_*` tools.

## Configure

Add to `~/.claude/rules.md` or project rules:

```markdown
- DejaVu memory rules (from pi-dejavu-memory extension)
```

Set your MCP endpoint (new path, or legacy `pi-nocturne-memory` path):

```json
{ "mcpUrl": "https://dejavu.example.com/mcp", "mcpHeaders": { "CF-Access-Client-Id": "<service-token-id>", "CF-Access-Client-Secret": "<service-token-secret>" } }
```

For servers behind Cloudflare Access (e.g. dejavu.example.com), pass the **service token** headers instead of `mcpAuth`:

```json
{
  "mcpUrl": "https://dejavu.example.com/mcp",
  "mcpHeaders": {
    "CF-Access-Client-Id": "your-client-id",
    "CF-Access-Client-Secret": "your-client-secret"
  }
}
```

`mcpHeaders` is merged into every MCP request; `mcpAuth` (Authorization) can be combined if the server also accepts it.

## How It Works

1. **SessionStart Hook** — triggers boot at session start
2. **Agent calls `noc_boot`** — loads `system://boot`, `system://recent/5`, `system://triggers`, then best-effort `system://briefing`
3. **Agent reads `system://focus`** — resume active working trees (`recent` already covered by boot/briefing)
4. **Global Rules** — memory operation rules injected every session
5. **Agent uses memory tools** — read/create/update/delete based on rules

## Why noc_* (not nocturne_*)?

Some agents probe for `read_mcp_resource` before reaching for a memory tool, which wastes a round trip ([upstream issue #32](https://github.com/Dataojitori/nocturne_memory/issues/32)). The `noc_boot` / `noc_read` naming, plus the boot protocol text in the rules, steers models to the right tool explicitly — no resource shim needed.

## License

MIT

## Related

- [DejaVu](https://github.com/RealAlexandreAI/DejaVu) — the MCP memory server this extension talks to
- [dsh-dejavu-memory](https://github.com/RetiredPhysicist/dsh-plugins/tree/main/packages/dsh-dejavu-memory) — same memory tools for dsh
- [nocturne_memory](https://github.com/Dataojitori/nocturne_memory) — upstream project

