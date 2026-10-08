# UMA Status & Roadmap (2026-10-08)

One-page synthesis: what exists today (the L0 baseline), what is already
implemented from `docs/proposals/01–05` (the L1–L5 cognitive roadmap), and
what comes next. Test gate at the time of writing: **158 Rust + 21
TypeScript tests, 0 warnings**; repo at `github.com/mastnacek/ai-memory`.

---

## 1. Baseline — what UMA is today (L0: the memory core)

**Architecture**: Vertical Slice Architecture. Rust kernel (`uma/uma-core`)
+ CLI (`uma/uma-cli`, composition root dispatch only) + Pi agent TypeScript
plugin (`uma-pi-extension`). Storage is OKF v0.2: Markdown + YAML frontmatter
in `.uma/` (project) and `%APPDATA%/uma/uma/data/global` (global), ULIDs,
git-tracked, with a central SQLite FTS5 index (WAL, busy_timeout) and an
OpenRouter embeddings cache.

**Kernel modules**: domain (Fact/Validity/ULID), serialization (roundtrip),
store (write/read/list/supersede + cross-project lookup), indexer, search
(BM25 / semantic / hybrid RRF with visible degradation), similarity,
consolidate, skill (templates expand, never execute), timeline, health
(doctor, read-only), embeddings, secrets (credential gate), fastbrain
(System-1 triage), risk (pain score), sources (pi/claude session readers).

**CLI — 19 commands**: write, read, list, search, supersede, migrate,
consolidate, skill, mcp (read-only unless `--allow-writes`), timeline,
export (OKF bundle), doctor, sync (git over the global store only), scopes,
sessions, import (session→memory candidates, proposals only), secrets scan,
recall check, risk pain.

**Pi plugin — 7 tools** (write, read, list, search, supersede, consolidate,
skill) + 3 hooks (fail-closed approval gate; fastbrain recall gate on
`before_agent_start`; immune interceptor warn-mode on `tool_call`) + the
approval modal (i18n cs/en, since/stale-after/template/supersedes) + `/uma`
commands (search, list, read, reindex, timeline, export, doctor, lang,
auto-approve, recall, judge) with completions.

**Memory**: ~30 active facts (project + global), 5 supersession chains,
zombie-free per the last consolidator run; every mutation passes the modal.

**Recorded design decisions** (selection): VSA everywhere; consolidation
proposes, never applies; skills expand, never execute; MCP read-only by
default; sync is git over the global store; degradation belongs in the
return value, not a log line; health checks never repair; modal round-trip
preserves the whole reviewed contract; **auto-blocking is reserved for
deterministic contracts, probabilistic verdicts may only warn**
(`01M4EP264QYX1FA23JQKT3GFQM`).

---

## 2. Proposals 01–05: what is implemented, what is not

| Proposal | Shipped | Remaining |
| :--- | :--- | :--- |
| **01 Cognitive immune system** (interceptor) | **Warn-mode**: before every `write`/`edit`, the interceptor warns from (a) the file's pain score and (b) vocabulary overlap against active rules (L1 cache, 10-min TTL). Fail-silent, 5s bounds, UI-only. | **Block-mode** — deliberately NOT implemented: gated on P03a deterministic contracts, never on probabilistic verdicts (recorded decision). Jev-per-edit advisory not yet wired. |
| **02 Shadow brain** (zero-latency mining) | Nothing yet. Telemetry triggers studied; S9 session mining is the offline precursor. | SPAI-005: turn_end/tool_result capture → `.uma/.staging/` drafts → `ctx.ui.setStatus` indicator → `/uma review` batch consent. |
| **03 Plasticity & contracts** | Nothing yet (the approval-gate test-locked pattern is the design precedent). | SPAI-003: `contract:` frontmatter → ast-grep export → generated `architecture_invariants.rs` + `uma contracts check`; block-mode only for contract-backed rules. SPAI-004: `plasticity:` + `saliency:` schema v0.3; doctor zombie report proposes, never auto-archives. |
| **04 Council & prudence** | **Pain score** (the deterministic Pillar II): kernel `risk/` + `uma risk pain` — corrections ×15 (cap 45), reverts ×20 (cap 60), churn ×2 (cap 20); bands low/medium/critical with band-specific guidance. Live: all repo files currently low. | Pillar I skeptic (on-demand only), Pillar III debt ledger, Pillar IV epistemic-humility mode — SPAI-006 (deferred). |
| **05 Muscle / priming / saliency / dreaming** | Nothing yet. `saliency` is folded into the SPAI-004 schema pass. | Muscle routines must be operator-curated (auto-synthesis proposal-only — they collide with the "skills expand, never execute" decision); priming graph and `doctor --dream` deferred. |

**Summary of the pyramid**: L0 is complete and hardened. L1 exists in its
advisory half (pain score + rule warnings). L2–L5 are designed, sequenced,
and tracked, with L3 (contracts) as the keystone that unlocks L1's full
reflex.

---

## 3. Baseline numbers

- 158 Rust tests (64 CLI + 90 core + 4 integration) + 21 TypeScript tests, 0 warnings
- CLI: 19 commands; kernel: 16 modules; plugin: 7 tools, 3 hooks, 3 slash-surface groups
- Roadmap tracking: SPAI-001 (native Jev transport), SPAI-003 (contracts),
  SPAI-004 (OKF v0.3 schema), SPAI-005 (shadow worker), SPAI-006 (deferred tail)
- Latest commit: see `git log -1` (chain continues from `12e992e`)

---

## 4. Build order (agreed)

1. **P03a contracts** — the keystone; unlocks interceptor block-mode.
2. **P03b+P05a OKF v0.3** — one schema pass for plasticity + saliency.
3. **P02 shadow worker** — staging + review UX.
4. Interceptor block-mode for contract-backed rules; P04 skeptic/ledger.
5. Deferred tail (muscle curation, priming, dreaming).

Standing invariants every phase must respect: proposal-only mutations
through the modal; degradation visible, never silent; deterministic floor
testable offline; probabilistic judgment advises, deterministic facts decide.