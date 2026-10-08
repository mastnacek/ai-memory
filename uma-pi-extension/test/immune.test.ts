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

test("an edit touching a rule's vocabulary warns once", () => {
  const added = `
    // feature slice calling another slice directly
    use crate::slices::doctor::checks::inspect_index;
    import slices into each other instead of the shared kernel
  `;
  const warnings = assessEdit(pain(0, "low"), [vsaRule], added);
  assert.equal(warnings.length, 1);
  assert.ok(warnings[0].includes(vsaRule.id));
  assert.ok(warnings[0].includes(vsaRule.title));
});

test("rule warnings stop at one per edit", () => {
  const secondRule: RuleL1 = { ...vsaRule, id: "01OTHER", title: "Another overlapping rule about slices and kernel" };
  const added = "slices kernel slices kernel slices kernel import shared";
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
