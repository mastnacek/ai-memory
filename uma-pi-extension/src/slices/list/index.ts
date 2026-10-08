import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import type { ExtensionState } from "../../shared/types.js";
import { executeUma } from "../../shared/client.js";

export function registerListTool(pi: ExtensionAPI, _state: ExtensionState): void {
  pi.registerTool({
    name: "uma_list",
    label: "UMA List",
    description: "List memory facts from UMA, optionally filtered by scope and fact type.",
    parameters: Type.Object({
      scope: Type.Optional(
        Type.String({
          description: "Scope: 'global' or project name. Defaults to current git repo.",
        })
      ),
      type: Type.Optional(
        Type.String({
          description: "Filter by type (decision, preference, note, pattern, etc.).",
        })
      ),
    }),
    execute: async (_toolCallId, params, _signal, _onUpdate, ctx) => {
      const args = ["list"];
      if (params.scope) args.push("--scope", params.scope);
      if (params.type) args.push("--type", params.type);
      return executeUma(ctx.cwd, args);
    },
  });
}
