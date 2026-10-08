import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import type { ExtensionState } from "../../shared/types.js";
import { executeUma } from "../../shared/client.js";

export function registerConsolidateTool(pi: ExtensionAPI, _state: ExtensionState): void {
  pi.registerTool({
    name: "uma_consolidate",
    label: "UMA Consolidate",
    description:
      "Review UMA memory for near-duplicate facts (proposed merges) and opposing facts (contradiction flags). Read-only: it never modifies memory. After reviewing a proposal, apply it yourself with uma_supersede (retire the losing fact) or uma_write (state the merged result).",
    parameters: Type.Object({
      scope: Type.Optional(
        Type.String({
          description: "Scope to analyse: 'global' or a project name. Defaults to the current project.",
        })
      ),
      type: Type.Optional(
        Type.String({ description: "Restrict to one fact type (decision, preference, pattern, ...)." })
      ),
      threshold: Type.Optional(
        Type.Number({ description: "Similarity threshold 0.0-1.0 above which a merge is proposed (default 0.55)." })
      ),
      includeDeprecated: Type.Optional(
        Type.Boolean({ description: "Also analyse deprecated facts (default false)." })
      ),
    }),
    execute: async (_toolCallId, params, _signal, _onUpdate, ctx) => {
      const args = ["consolidate", "--json"];
      if (params.scope) args.push("--scope", params.scope);
      if (params.type) args.push("--type", params.type);
      if (typeof params.threshold === "number") args.push("--threshold", String(params.threshold));
      if (params.includeDeprecated) args.push("--include-deprecated");
      return executeUma(ctx.cwd, args, '{"scanned":0,"duplicate_groups":[],"contradictions":[]}');
    },
  });
}
