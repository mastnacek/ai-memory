import assert from "node:assert/strict";
import { test } from "node:test";
import { evaluateApprovalGate, MEMORY_MUTATING_TOOLS } from "../src/hooks/approval_gate.ts";

/** Minimal fakes: the gate only reads `mode`, `hasUI`, and `config.autoApprove`. */
const ctx = (mode: string, hasUI: boolean) => ({ mode, hasUI }) as never;
const state = (autoApprove: boolean) => ({ config: { lang: "cs", autoApprove } }) as never;
const call = (toolName: string) => ({ type: "tool_call", toolCallId: "1", toolName, input: {} }) as never;

test("read-only tools are never gated", () => {
  // uma_consolidate belongs here: it proposes merges/contradictions but never
  // writes, so gating it would make approval routine instead of meaningful.
  for (const tool of ["uma_read", "uma_list", "uma_search", "uma_consolidate", "bash", "read"]) {
    assert.equal(evaluateApprovalGate(call(tool), ctx("print", false), state(false)), undefined);
  }
});

test("the gated set is exactly the memory-mutating tools", () => {
  assert.deepEqual([...MEMORY_MUTATING_TOOLS].sort(), ["uma_supersede", "uma_write"]);
});

test("TUI with auto-approve off allows the call so the modal can open", () => {
  assert.equal(evaluateApprovalGate(call("uma_write"), ctx("tui", true), state(false)), undefined);
});

test("TUI with auto-approve on allows the call", () => {
  assert.equal(evaluateApprovalGate(call("uma_write"), ctx("tui", true), state(true)), undefined);
});

test("non-interactive mode fails closed for every mutating tool", () => {
  for (const tool of MEMORY_MUTATING_TOOLS) {
    const result = evaluateApprovalGate(call(tool), ctx("print", false), state(false));
    assert.ok(result?.block, `${tool} must be blocked without a UI`);
    assert.match(result.reason ?? "", /approval/);
  }
});

test("auto-approve on permits non-interactive writes", () => {
  assert.equal(evaluateApprovalGate(call("uma_write"), ctx("print", false), state(true)), undefined);
});

test("hasUI true in a non-tui mode is still not an approval surface", () => {
  // RPC reports hasUI but cannot render custom components, so it must block.
  const result = evaluateApprovalGate(call("uma_supersede"), ctx("rpc", true), state(false));
  assert.ok(result?.block);
});
