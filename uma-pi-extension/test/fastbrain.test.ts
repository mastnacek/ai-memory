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
  scope: "project:ai-memory",
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
  assert.ok(report.length < 160, `too long: ${report.length}`);
  assert.ok(!report.includes("\n"), report);
});
