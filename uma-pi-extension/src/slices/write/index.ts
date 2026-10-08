import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "typebox";
import type { ExtensionState, MemoryProposal } from "../../shared/types.js";
import { executeUma } from "../../shared/client.js";
import { showProposalModal } from "../../shared/modal.js";

export function registerWriteTool(pi: ExtensionAPI, state: ExtensionState): void {
  pi.registerTool({
    name: "uma_write",
    label: "UMA Write",
    description: "Store a new structured memory fact, decision, preference, skill, or note in UMA memory.",
    parameters: Type.Object({
      title: Type.String({ description: "One-line descriptive title of the fact or decision." }),
      body: Type.String({ description: "Detailed Markdown body / content explaining context, rule, and consequences." }),
      type: Type.Optional(
        Type.String({
          description: "Fact type: decision, preference, fact, skill, correction, note, pattern, reference, task (default: note).",
        })
      ),
      scope: Type.Optional(
        Type.String({
          description: "Scope: 'global' or project name. Defaults to current git repo.",
        })
      ),
      tags: Type.Optional(
        Type.Array(Type.String(), {
          description: "List of tags (e.g. ['architecture', 'vsa', 'rust']).",
        })
      ),
      template: Type.Optional(
        Type.String({
          description:
            "Only for type 'skill': an invocation template with {{placeholders}}, e.g. \"docker build -t {{tag}} .\". Stored as data and never executed.",
        })
      ),
      staleAfter: Type.Optional(
        Type.String({
          description:
            "When the claim stops being trusted without re-verification. ISO 8601 timestamp or a bare date (2026-12-31).",
        })
      ),
    }),
    execute: async (_toolCallId, params, _signal, _onUpdate, ctx) => {
      const initialProposal: MemoryProposal = {
        title: params.title,
        body: params.body,
        type: params.type || "note",
        scope: params.scope || "project",
        tags: params.tags || [],
        template: params.template,
        stale_after: params.staleAfter,
      };

      // In interactive TUI mode (and when autoApprove is false), show the modal proposal window
      let approvedProposal = initialProposal;
      if (!state.config.autoApprove && ctx.mode === "tui" && ctx.hasUI) {
        const modalResult = await showProposalModal(ctx, initialProposal, state.config.lang);
        if (modalResult.action === "rejected") {
          return {
            content: [{ type: "text", text: "Memory proposal was cancelled/rejected by the user." }],
            details: { rejected: true },
          };
        }
        approvedProposal = modalResult.proposal;
      }

      const args = ["write", "--title", approvedProposal.title, "--body", approvedProposal.body];
      if (approvedProposal.type) {
        args.push("--type", approvedProposal.type);
      }
      if (approvedProposal.scope) {
        args.push("--scope", approvedProposal.scope);
      }
      if (approvedProposal.tags.length > 0) {
        args.push("--tags", approvedProposal.tags.join(","));
      }
      if (approvedProposal.template) {
        args.push("--template", approvedProposal.template);
      }
      if (approvedProposal.stale_after) {
        args.push("--stale-after", approvedProposal.stale_after);
      }

      return executeUma(ctx.cwd, args);
    },
  });
}
