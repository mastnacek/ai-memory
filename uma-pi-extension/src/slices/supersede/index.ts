import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import type { ExtensionState, MemoryProposal } from "../../shared/types.js";
import { executeUma, readFactJson } from "../../shared/client.js";
import { showProposalModal } from "../../shared/modal.js";
import { scanProposal, refusalText } from "../../shared/secrets_client.js";

export function registerSupersedeTool(pi: ExtensionAPI, state: ExtensionState): void {
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
      template: Type.Optional(
        Type.String({
          description:
            "Invocation template for 'skill' facts; defaults to the predecessor's template. Placeholders use {{name}} and are never executed.",
        })
      ),
      staleAfter: Type.Optional(
        Type.String({
          description:
            "When the claim needs re-verification; defaults to the predecessor's stale_after. ISO 8601 timestamp or a bare date.",
        })
      ),
      since: Type.Optional(
        Type.String({
          description:
            "When the revised claim started to hold; defaults to the predecessor's since. ISO 8601 timestamp or a bare date.",
        })
      ),
    }),
    execute: async (_toolCallId, params, _signal, _onUpdate, ctx) => {
      // Resolve the predecessor so the modal shows its real type/scope/tags and
      // so omitted arguments inherit from it, matching the CLI's behaviour.
      const predecessor = await readFactJson(ctx.cwd, params.oldId);

      const proposal: MemoryProposal = {
        title: params.title,
        body: params.body,
        type: params.type || predecessor?.type || "note",
        scope: params.scope || predecessor?.scope || "project",
        tags: params.tags && params.tags.length > 0 ? params.tags : predecessor?.tags ?? [],
        supersedes: params.oldId,
        // Carried into the proposal so the reviewer SEES the template being set
        // or replaced. Omitted means the CLI inherits the predecessor's.
        template: params.template,
        stale_after: params.staleAfter,
        since: params.since,
      };

      // Secret gate BEFORE the modal, same as the write tool: a revision
      // carrying a credential never reaches the operator as saveable.
      const scanText = [proposal.title, proposal.body, proposal.template]
        .filter(Boolean)
        .join("\n");
      const scan = await scanProposal(ctx.cwd, scanText);
      if (scan.blocked) {
        return {
          content: [{ type: "text", text: refusalText(scan) }],
          details: { rejected: true, secretFindings: scan.findings },
        };
      }

      // Same approval contract as uma_write: review the revision before it is stored.
      let approved = proposal;
      if (!state.config.autoApprove && ctx.mode === "tui" && ctx.hasUI) {
        const result = await showProposalModal(ctx, proposal, state.config.lang);
        if (result.action === "rejected") {
          return {
            content: [
              { type: "text", text: "Supersede proposal was cancelled/rejected by the user." },
            ],
            details: { rejected: true },
          };
        }
        approved = result.proposal;
      }

      const args = ["supersede", params.oldId, "--title", approved.title, "--body", approved.body];
      if (params.description) args.push("--desc", params.description);
      // Use the approved value, never the raw parameter: if the modal ever gains
      // template editing, the reviewed value must be the one that is stored.
      if (approved.template) args.push("--template", approved.template);
      if (approved.stale_after) args.push("--stale-after", approved.stale_after);
      if (approved.since) args.push("--since", approved.since);
      if (approved.type) args.push("--type", approved.type);
      if (approved.scope) args.push("--scope", approved.scope);
      if (approved.tags.length > 0) args.push("--tags", approved.tags.join(","));
      return executeUma(ctx.cwd, args);
    },
  });
}
