import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { loadConfig } from "./src/shared/config.js";
import { createExtensionState } from "./src/shared/state.js";
import { registerWriteTool } from "./src/slices/write/index.js";
import { registerReadTool } from "./src/slices/read/index.js";
import { registerListTool } from "./src/slices/list/index.js";
import { registerSearchTool } from "./src/slices/search/index.js";
import { registerSupersedeTool } from "./src/slices/supersede/index.js";
import { registerCommands } from "./src/slices/commands/index.js";
import { evaluateApprovalGate } from "./src/hooks/approval_gate.js";

export default function umaExtension(pi: ExtensionAPI): void {
  // 1. Guard against subagent recursion
  if (process.env.PI_SUBAGENT === "true" || Boolean(process.env.PI_CHILD_SESSION)) return;

  // 2. Unsubscribers drain on session_shutdown
  const unsubscribers: Array<() => void> = [];
  const track = (res: unknown): void => {
    if (typeof res === "function") unsubscribers.push(res as () => void);
  };

  // 3. Initialize state & config
  const initial = loadConfig(process.cwd());
  const state = createExtensionState(initial.config, initial.globalFile);
  state.unsubscribers = unsubscribers;

  // 4. Session lifecycle hooks
  track(
    pi.on("session_start", (_event, ctx) => {
      const refreshed = loadConfig(ctx.cwd);
      state.config = refreshed.config;
      state.globalConfigFile = refreshed.globalFile;
    })
  );

  pi.on("session_shutdown", () => {
    while (unsubscribers.length > 0) {
      unsubscribers.pop()?.();
    }
  });

  // 5. Guard hook: fail closed on memory mutations without an approval UI.
  track(pi.on("tool_call", (event, ctx) => evaluateApprovalGate(event, ctx, state)));

  // 6. Register feature slices — one call per slice, no aggregator layer.
  registerWriteTool(pi, state);
  registerReadTool(pi, state);
  registerListTool(pi, state);
  registerSearchTool(pi, state);
  registerSupersedeTool(pi, state);
  registerCommands(pi, state);
}
