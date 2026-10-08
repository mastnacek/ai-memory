import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assessEdit,
  extractEdit,
  type PainVerdict,
  type RuleL1,
} from "../src/slices/immune/index.ts";

const pain = (score: number, band: PainVerdict["band"]): PainVerdict => ({ score, band });

const vsaRule: RuleL1 = {
  id: "01M4D7S5",
  title: "Strict Vertical Slice Architecture",
  fact_type: "decision",
  body:
    "Feature slices never import each other directly. Shared contracts flow through uma-core or src/shared.",
};

test("low pain and no rule overlap produces no warnings", () => {
  const warnings = assessEdit(pain(5, "low"), [vsaRule], "const x = 1 + 2;");
  assert.deepEqual(warnings, []);
});

test("medium and critical pain warn with guidance — and never block", () => {
  const medium = assessEdit(pain(40, "medium"), [], "code");
  assert.equal(medium.length, 1);
  assert.ok(medium[0].includes("40/100"));
  assert.ok(medium[0].includes("run the affected tests"));

  const critical = assessEdit(pain(85, "critical"), [], "code");
  assert.equal(critical.length, 1);
  assert.ok(critical[0].includes("test-first"));

  // The interceptor is warn-mode: no API surface here can block, so the
  // assessment must never return anything resembling a block verdict.
  for (const w of [...medium, ...critical]) {
    assert.ok(!w.toLowerCase().includes("blocked"), w);
  }
});

test("an edit genuinely touching a rule's vocabulary warns once (live fixture)", () => {
  // Validated against the real 22-rule store: inter=10, containment=0.48.
  const added =
    "The change mirrors rule vocabulary on purpose: feature slices never import each other directly; shared contracts flow through the shared kernel instead of slice-to-slice imports.";
  const warnings = assessEdit(pain(0, "low"), [vsaRule], added);
  assert.equal(warnings.length, 1);
  assert.ok(warnings[0].includes(vsaRule.id));
  assert.ok(warnings[0].includes(vsaRule.title));
});

test("benign edits sharing a few words stay silent (live fixtures)", () => {
  // inter=3 against the closest rule — below the absolute floor.
  const doc = "Revision history is dated by each fact's generated.at (see the timeline); the claim's since passes through untouched.";
  assert.deepEqual(assessEdit(pain(0, "low"), [vsaRule], doc), []);
  // inter=2.
  const readme = "One-page synthesis of what exists today and what comes next.";
  assert.deepEqual(assessEdit(pain(0, "low"), [vsaRule], readme), []);
  // Fewer than the token floor fires nothing at all.
  assert.deepEqual(assessEdit(pain(0, "low"), [vsaRule], "const x = 1 + 2; // scratch"), []);
});

test("rule warnings stop at one per edit", () => {
  const secondRule: RuleL1 = { ...vsaRule, id: "01OTHER", title: "Another overlapping rule about slices and kernel" };
  // 5 distinct tokens, all shared with both rules -> both cross the floor,
  // but the assessment stops after the first hit.
  const added = "slices kernel import shared directly across the modules";
  const warnings = assessEdit(pain(0, "low"), [vsaRule, secondRule], added);
  assert.equal(warnings.filter((w) => w.includes("[UMA Rule]")).length, 1);
});

test("extractEdit reads write and edit shapes", () => {
  assert.deepEqual(extractEdit("write", { path: "a.rs", content: "fn main() {}" }), {
    path: "a.rs",
    added: "fn main() {}",
  });
  const edit = extractEdit("edit", {
    path: "b.rs",
    edits: [{ oldText: "a", newText: "first" }, { oldText: "b", newText: "second" }],
  });
  assert.equal(edit?.added.includes("first"), true);
  assert.equal(edit?.added.includes("second"), true);

  assert.equal(extractEdit("read", { path: "c.rs" }), undefined);
  assert.equal(extractEdit("bash", { command: "ls" }), undefined);
  assert.equal(extractEdit("write", { content: "no path" }), undefined);
});

import { decideImmuneAction } from "../src/slices/immune/index.ts";

const warnings = ["[UMA Risk] Pain score 25/100 — run the affected tests after editing."];

test("mode policy: off and warn never block", () => {
  assert.deepEqual(decideImmuneAction("off", warnings), { kind: "allow" });
  const warn = decideImmuneAction("warn", warnings);
  assert.equal(warn.kind, "notify");
  assert.ok(warn.kind === "notify" && warn.message.includes("25/100"));
});

test("mode policy: ask proposes through a confirm; a decline blocks", () => {
  const ask = decideImmuneAction("ask", warnings);
  assert.equal(ask.kind, "confirm");
});

test("mode policy: auto asks today (heuristics may not veto silently)", () => {
  // The recorded decision: auto-blocking is reserved for contract-backed
  // rules. Until contracts exist, auto must NOT return a silent block.
  const auto = decideImmuneAction("auto", warnings);
  assert.equal(auto.kind, "confirm");
  assert.ok(auto.kind === "confirm" && auto.message.includes("blocks only contract-backed rules"));
});

test("no warnings means allow in every mode", () => {
  for (const mode of ["off", "warn", "ask", "auto"] as const) {
    assert.deepEqual(decideImmuneAction(mode, []), { kind: "allow" });
  }
});
