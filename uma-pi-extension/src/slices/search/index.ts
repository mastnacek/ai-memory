import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import type { ExtensionState } from "../../shared/types.js";
import { executeUma } from "../../shared/client.js";

export function registerSearchTool(pi: ExtensionAPI, _state: ExtensionState): void {
  pi.registerTool({
    name: "uma_search",
    label: "UMA Search",
    description: "Search UMA memory facts using hybrid BM25 keyword matching and semantic vector search.",
    parameters: Type.Object({
      query: Type.String({ description: "Search query keywords or semantic concept." }),
      mode: Type.Optional(
        Type.String({
          description: "Search mode: 'keyword' (BM25), 'semantic' (Vector), or 'hybrid' (default).",
        })
      ),
      scope: Type.Optional(
        Type.String({
          description: "Scope to search: 'global' or project name. Defaults to searching both.",
        })
      ),
      type: Type.Optional(
        Type.String({
          description: "Filter by fact type (decision, preference, pattern, skill, note, etc.).",
        })
      ),
      includeDeprecated: Type.Optional(
        Type.Boolean({
          description: "Include deprecated/superseded facts in results (default: false).",
        })
      ),
      asOf: Type.Optional(
        Type.String({
          description: "Point-in-time search: only facts active at this ISO 8601 datetime (e.g. '2026-01-15T00:00:00Z').",
        })
      ),
      limit: Type.Optional(
        Type.Number({
          description: "Maximum number of results to return (default: 10).",
        })
      ),
    }),
    execute: async (_toolCallId, params, _signal, _onUpdate, ctx) => {
      const args = ["search", params.query];
      if (params.mode) args.push("--mode", params.mode);
      if (params.scope) args.push("--scope", params.scope);
      if (params.type) args.push("--type", params.type);
      if (params.includeDeprecated) args.push("--include-deprecated");
      if (params.asOf) args.push("--as-of", params.asOf);
      if (params.limit) args.push("--limit", String(params.limit));
      return executeUma(ctx.cwd, args, "No matching facts found.");
    },
  });
}
