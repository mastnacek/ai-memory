import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import type { ExtensionState } from "../../shared/types.js";
import { executeUma } from "../../shared/client.js";

export function registerSkillTool(pi: ExtensionAPI, _state: ExtensionState): void {
  pi.registerTool({
    name: "uma_skill_invoke",
    label: "UMA Skill Invoke",
    description:
      "Expand a stored UMA skill template into a concrete command. Returns the command text ONLY — it never executes anything. Run the returned command yourself with your shell tool so your own approval applies. Create skills with uma_write (type 'skill' plus a template).",
    parameters: Type.Object({
      name: Type.String({ description: "Skill name or fact ID (ULID)." }),
      set: Type.Optional(
        Type.Array(Type.String(), {
          description: "Placeholder values as 'key=value' strings, e.g. ['tag=v1'].",
        })
      ),
      scope: Type.Optional(
        Type.String({ description: "Scope to search first: 'global' or a project name." })
      ),
    }),
    execute: async (_toolCallId, params, _signal, _onUpdate, ctx) => {
      const args = ["skill", "invoke", params.name, "--json"];
      for (const pair of params.set ?? []) {
        args.push("--set", pair);
      }
      if (params.scope) {
        args.push("--scope", params.scope);
      }
      return executeUma(ctx.cwd, args);
    },
  });
}
