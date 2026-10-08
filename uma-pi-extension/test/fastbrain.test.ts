import assert from "node:assert/strict";
import { test } from "node:test";
import { buildRecallMessage, type RecallVerdict } from "../src/slices/fastbrain/policy.ts";

const verdict = (facts: RecallVerdict["facts"]): RecallVerdict => ({
  search: true,
  judged_by: "jev",
  fact_types: ["decision"],
  recalled: facts.length,
  facts,
  note: null,
});

const fact = {
  id: "01TEST",
  title: "Sync is git-based over the global store only",
  fact_type: "decision",
  // The CLI serializes the Scope enum in serde's external wire form.
  scope: { Project: "ai-memory" } as const,
  score: 0.9,
  snippet: "Sync is git-based…",
  tags: ["sync"],
};

test("no trigger means no injected message", () => {
  assert.equal(buildRecallMessage(verdict([]), "en"), undefined);
  assert.equal(
    buildRecallMessage({ ...verdict([fact]), search: false }, "en"),
    undefined,
  );
});

test("a trigger injects one hidden message with the facts", () => {
  const result = buildRecallMessage(verdict([fact]), "en");
  assert.ok(result);
  assert.equal(result.message?.customType, "uma-recall");
  // Hidden: display false keeps the transcript clean; the model still reads it.
  assert.equal(result.message?.display, false);
  assert.ok(String(result.message?.content).includes("Sync is git-based"));
});

test("the injected header follows the configured language", () => {
  const en = buildRecallMessage(verdict([fact]), "en");
  const cs = buildRecallMessage(verdict([fact]), "cs");
  assert.ok(String(en?.message?.content).includes("Memory relevant"));
  assert.ok(String(cs?.message?.content).includes("Paměť k tomuto úkolu"));
});

import { formatGateDecision } from "../src/slices/fastbrain/policy.ts";

test("the decision report shows ask, verdict, and next action", () => {
  const report = formatGateDecision(
    "Can we refactor the sync slice like we decided yesterday?",
    verdict([fact]),
    "en",
  );
  assert.ok(report.includes('Judge jev · "Can we refactor the sync slice'), `asked: ${report}`);
  assert.ok(report.includes("trigger (decision)"), `verdict: ${report}`);
  assert.ok(report.includes("injecting 1 fact(s)"), `next: ${report}`);
});

test("a no-trigger verdict reports memory staying closed", () => {
  const report = formatGateDecision(
    "thanks, commit it",
    { ...verdict([fact]), search: false, fact_types: [], facts: [] },
    "en",
  );
  assert.ok(report.includes("no trigger"), report);
  assert.ok(report.includes("memory stays closed"), report);
});

test("a degraded judge verdict carries the warning", () => {
  const report = formatGateDecision(
    "why did we pick pnpm?",
    { ...verdict([fact]), note: "Jev unavailable (401); offline verdict" },
    "en",
  );
  assert.ok(report.includes("Jev unavailable"), report);
});

test("long prompts are elided and whitespace collapsed", () => {
  const report = formatGateDecision("word ".repeat(40), verdict([fact]), "en");
  // The decision line stays short even when the fact list follows.
  assert.ok(report.split("\n")[0].length < 160, `decision line too long: ${report}`);
});

test("the injected facts are listed under the decision line", () => {
  const report = formatGateDecision(
    "How does the supersede chain preserve the origin?",
    verdict([fact, { ...fact, id: "01SECOND", title: "Second recalled fact" }]),
    "en",
  );
  const lines = report.split("\n");
  // Each fact takes two lines: numbered title + elided snippet.
  assert.ok(lines[0].includes("injecting 2 fact(s)"), lines[0]);
  assert.ok(lines[1].includes("1. [decision] Sync is git-based"), lines[1]);
  assert.ok(lines[1].includes("project:ai-memory"), lines[1]);
  assert.ok(lines[3].includes("2. [decision] Second recalled fact"), lines[3]);
});

test("no-trigger and zero-fact verdicts list no facts", () => {
  const closed = formatGateDecision(
    "thanks",
    { ...verdict([fact]), search: false, fact_types: [], facts: [] },
    "en",
  );
  assert.ok(!closed.includes("\n   1."), closed);
  assert.ok(!formatGateDecision("why?", verdict([]), "en").includes("\n   1."));
});

import { buildTranslateContext } from "../src/slices/translate/prompt.ts";
import { stringsFor } from "../src/shared/i18n.ts";
import { renderProposalView } from "../src/shared/modal_renderer.ts";
import { visibleWidth } from "@earendil-works/pi-tui";

test("translate context is a real Context, not a bare array", () => {
  const ctx = buildTranslateContext("## Rule\n- use ULID 01M4DQ9C37YR0J7ZG358DQQA6F");
  // A bare message array (or a timestamp-less message) passes a cast but
  // fails the real Message contract — the bug that made translation
  // silently unavailable.
  assert.ok(Array.isArray(ctx.messages) && ctx.messages.length === 1);
  assert.ok(ctx.systemPrompt.includes("ULID"));
  assert.ok(ctx.systemPrompt.includes("#tag"));
  assert.ok(ctx.systemPrompt.includes("Translate only the prose"));
  assert.equal(ctx.messages[0].role, "user");
  assert.equal(ctx.messages[0].content, "## Rule\n- use ULID 01M4DQ9C37YR0J7ZG358DQQA6F");
  assert.ok(typeof ctx.messages[0].timestamp === "number");
});

const demoTheme = { fg: (_c: string, t: string) => t, bg: (_c: string, t: string) => t, bold: (t: string) => t } as any;
const longBody = Array.from({ length: 30 }, (_, i) => `Line ${i + 1} of the fact body.`).join("\n");
const demoProposal = {
  title: "Full text demo",
  body: longBody,
  type: "decision",
  scope: "global",
  tags: ["demo"],
};
const s = stringsFor("cs");
const base = { proposal: demoProposal, projectName: "ai-memory", editField: null as null, editorLines: [] as string[], width: 80, theme: demoTheme, s };

test("full view renders a scroll window with a position marker", () => {
  const lines = renderProposalView({ ...base, actionIndex: 0, bodyText: demoProposal.body, fullView: { scrollOffset: 10, viewportLines: 16 } });
  const joined = lines.join("\n");
  assert.ok(joined.includes("Line 11 "), `scroll window starts at offset: ${joined}`);
  assert.ok(joined.includes("Line 26"), "window covers offset+16");
  assert.ok(!joined.includes("Line 31"), "window does not run past the body");
  assert.ok(joined.includes("[11–26/30]"), `position marker: ${joined}`);
  assert.ok(lines.every((l) => visibleWidth(l) <= 80));
});

test("preview shows the translation note without touching the title", () => {
  const lines = renderProposalView({ ...base, actionIndex: 0, bodyText: "Přeložený text.", bodyNote: s.translatedNote });
  assert.ok(lines.join("\n").includes("ukládá se ORIGINÁL"));
  assert.ok(lines.join("\n").includes("Full text demo"));
  assert.ok(lines.join("\n").includes("Přeložený text."));
});

test("preview caps at 8 lines and points to [v]", () => {
  const lines = renderProposalView({ ...base, actionIndex: 0, bodyText: demoProposal.body });
  const joined = lines.join("\n");
  assert.ok(joined.includes("+22 řádků"));
  assert.ok(joined.includes("[v] 📜"));
  assert.ok(!joined.includes("Line 9 "), "preview stops at 8 lines");
});
