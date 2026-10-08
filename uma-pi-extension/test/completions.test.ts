import assert from "node:assert/strict";
import { test } from "node:test";
import { getUmaCompletions } from "../src/slices/commands/complete.ts";
import type { PluginConfig } from "../src/shared/types.ts";

const state = (config: Partial<PluginConfig>) =>
  ({ config: { lang: "cs", autoApprove: false, recallGate: false, fastbrainJudge: "off", ...config } }) as never;

test("the /uma menu lists every subcommand the handler implements", () => {
  const items = getUmaCompletions("", state({}));
  const values = items.map((i) => i.value);
  for (const expected of ["search ", "list", "read ", "reindex", "timeline", "export ", "doctor", "lang ", "auto-approve ", "recall ", "judge "]) {
    assert.ok(values.includes(expected), `menu missing: ${expected}`);
  }
});

test("recall and judge complete with the current state marked", () => {
  const on = getUmaCompletions("recall ", state({ recallGate: true }));
  const onEntry = on.find((i) => i.value === "recall on");
  assert.ok(onEntry?.label.includes("✓"), "active option carries the checkmark");

  const off = getUmaCompletions("recall ", state({ recallGate: false }));
  const offEntry = off.find((i) => i.value === "recall off");
  assert.ok(offEntry?.label.includes("✓"));

  const jev = getUmaCompletions("judge ", state({ fastbrainJudge: "jev" }));
  assert.ok(jev.find((i) => i.value === "judge jev")?.label.includes("✓"));

  const markers = getUmaCompletions("judge ", state({ fastbrainJudge: "off" }));
  assert.ok(markers.find((i) => i.value === "judge off")?.label.includes("✓"));
});
test("immune mode completes with the active mode marked", () => {
  const items = getUmaCompletions("immune ", state({ immuneMode: "ask" }));
  const values = items.map((i) => i.value);
  for (const expected of ["immune off", "immune warn", "immune ask", "immune auto"]) {
    assert.ok(values.includes(expected), `missing: ${expected}`);
  }
  assert.ok(items.find((i) => i.value === "immune ask")?.label.includes("✓"));
  assert.ok(!items.find((i) => i.value === "immune warn")?.label.includes("✓"));
});
