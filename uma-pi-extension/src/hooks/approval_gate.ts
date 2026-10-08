import type { ExtensionContext, ToolCallEvent, ToolCallEventResult } from "@earendil-works/pi-coding-agent";
import type { ExtensionState } from "../shared/types.js";

/**
 * Tools that mutate the UMA store and therefore require an approval path.
 * Adding a future mutating tool means adding one entry here — the guard is
 * centralized rather than duplicated per tool.
 *
 * Deliberately excludes `uma_consolidate`: it only *proposes* merges and
 * contradictions and never writes. Gating a read would make approval routine.
 */
export const MEMORY_MUTATING_TOOLS = new Set(["uma_write", "uma_supersede"]);

/**
 * Fail-closed approval decision for a `tool_call`.
 *
 * Returns `undefined` to allow the call, or a block result. It is pure so it can
 * be unit tested; the composition root owns the `pi.on("tool_call", …)`
 * subscription and its unsubscribe.
 *
 * A mutation is allowed only when either an interactive approval UI exists (the
 * tool then shows its modal) or the operator explicitly opted into
 * auto-approval. This also covers nested calls issued from codemode scripts.
 */
export function evaluateApprovalGate(
  event: ToolCallEvent,
  ctx: ExtensionContext,
  state: ExtensionState
): ToolCallEventResult | undefined {
  if (!MEMORY_MUTATING_TOOLS.has(event.toolName)) return undefined;

  const hasApprovalUi = ctx.mode === "tui" && ctx.hasUI;
  if (hasApprovalUi || state.config.autoApprove) return undefined;

  return {
    block: true,
    reason:
      `${event.toolName} would write to UMA memory, but no interactive approval UI is ` +
      `available (mode=${ctx.mode}). Run interactively so the approval window can open, ` +
      `or enable auto-approval with "/uma auto-approve on" (add --global for all sessions).`,
  };
}
