import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import type { ExtensionState } from "../../shared/types.js";
import { executeUma } from "../../shared/client.js";

export function registerSupersedeTool(pi: ExtensionAPI, _state: ExtensionState): void {
  pi.registerTool({
    name: "uma_supersede",
    label: "UMA Supersede",
    description:
      "Replace an existing UMA memory fact with a revised version. The old fact is marked deprecated (not deleted) and the new fact chains back to it via 'supersedes'.",
    parameters: Type.Object({
      oldId: Type.String({ description: "ULID of the predecessor fact to supersede." }),
      title: Type.String({ description: "Title of the new revised fact." }),
      body: Type.String({ description: "Markdown body of the new revised fact." }),
      description: Type.Optional(
        Type.String({ description: "Optional one-line description (OKF format)." })
      ),
      type: Type.Optional(
        Type.String({ description: "Fact type; defaults to the predecessor's type if omitted." })
      ),
      scope: Type.Optional(
        Type.String({ description: "Scope; defaults to the predecessor's scope if omitted." })
      ),
      tags: Type.Optional(
        Type.Array(Type.String(), { description: "Tags; defaults to the predecessor's tags if omitted." })
      ),
    }),
    execute: async (_toolCallId, params, _signal, _onUpdate, ctx) => {
      const args = ["supersede", params.oldId, "--title", params.title, "--body", params.body];
      if (params.description) args.push("--desc", params.description);
      if (params.type) args.push("--type", params.type);
      if (params.scope) args.push("--scope", params.scope);
      if (params.tags && params.tags.length > 0) args.push("--tags", params.tags.join(","));
      return executeUma(ctx.cwd, args);
    },
  });
}
