import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { Editor, type EditorTheme, Key, matchesKey } from "@earendil-works/pi-tui";
import * as path from "node:path";
import type { MemoryProposal, ProposalResult } from "./types.js";
import { stringsFor } from "./i18n.js";
import { renderProposalView } from "./modal_renderer.js";

type EditField = "title" | "body" | "tags" | null;

function resolveProjectName(cwd: string): string {
  const base = path.basename(cwd);
  return base || "project";
}

export async function showProposalModal(
  ctx: ExtensionContext,
  initialProposal: MemoryProposal,
  lang = "cs"
): Promise<ProposalResult> {
  const s = stringsFor(lang);
  const projectName = resolveProjectName(ctx.cwd);

  // Fail closed: with no interactive UI there is nobody to approve, so refuse.
  // The `tool_call` approval gate blocks this path first; this is defence in
  // depth in case the modal is ever reached without a UI.
  if (ctx.mode !== "tui" || !ctx.hasUI) {
    return { action: "rejected", proposal: initialProposal };
  }

  // Normalize initial scope
  const normalizedInitialScope =
    initialProposal.scope.toLowerCase() === "global" ? "global" : projectName;

  const result = await ctx.ui.custom<ProposalResult | null>((tui, theme, _kb, done) => {
    const proposal: MemoryProposal = {
      title: initialProposal.title,
      body: initialProposal.body,
      type: initialProposal.type,
      scope: normalizedInitialScope,
      tags: [...initialProposal.tags],
    };

    let actionIndex = 0;
    let editField: EditField = null;
    let cachedLines: string[] | undefined;

    const editorTheme: EditorTheme = {
      borderColor: (str: string) => theme.fg("accent", str),
      selectList: {
        selectedPrefix: (t: string) => theme.fg("accent", t),
        selectedText: (t: string) => theme.fg("accent", t),
        description: (t: string) => theme.fg("muted", t),
        scrollInfo: (t: string) => theme.fg("dim", t),
        noMatch: (t: string) => theme.fg("warning", t),
      },
    };

    const editor = new Editor(tui, editorTheme);

    function startEditing(field: EditField) {
      editField = field;
      if (field === "title") {
        editor.setText(proposal.title);
      } else if (field === "body") {
        editor.setText(proposal.body);
      } else if (field === "tags") {
        editor.setText(proposal.tags.join(", "));
      }
      refresh();
    }

    editor.onSubmit = (value) => {
      const trimmed = value.trim();
      if (editField === "title" && trimmed) {
        proposal.title = trimmed;
      } else if (editField === "body" && trimmed) {
        proposal.body = trimmed;
      } else if (editField === "tags") {
        proposal.tags = trimmed
          .split(",")
          .map((t) => t.trim())
          .filter(Boolean);
      }
      editField = null;
      editor.setText("");
      refresh();
    };

    function refresh() {
      cachedLines = undefined;
      tui.requestRender();
    }

    function toggleScope() {
      if (proposal.scope.toLowerCase() === "global") {
        proposal.scope = projectName;
      } else {
        proposal.scope = "global";
      }
      refresh();
    }

    function handleInput(data: string) {
      if (editField !== null) {
        if (matchesKey(data, Key.escape)) {
          editField = null;
          editor.setText("");
          refresh();
          return;
        }
        editor.handleInput(data);
        refresh();
        return;
      }

      const actionsCount = 6;

      if (matchesKey(data, Key.up)) {
        actionIndex = Math.max(0, actionIndex - 1);
        refresh();
        return;
      }
      if (matchesKey(data, Key.down)) {
        actionIndex = Math.min(actionsCount - 1, actionIndex + 1);
        refresh();
        return;
      }

      if (data === "e" || data === "E") {
        startEditing("title");
        return;
      }
      if (data === "b" || data === "B") {
        startEditing("body");
        return;
      }
      if (data === "t" || data === "T") {
        startEditing("tags");
        return;
      }
      if (data === "s" || data === "S") {
        toggleScope();
        return;
      }

      if (matchesKey(data, Key.enter)) {
        switch (actionIndex) {
          case 0:
            done({ action: "approved", proposal });
            break;
          case 1:
            startEditing("title");
            break;
          case 2:
            startEditing("body");
            break;
          case 3:
            startEditing("tags");
            break;
          case 4:
            toggleScope();
            break;
          case 5:
            done({ action: "rejected", proposal });
            break;
          default:
            break;
        }
        return;
      }

      if (matchesKey(data, Key.escape)) {
        done({ action: "rejected", proposal });
      }
    }

    function render(width: number): string[] {
      if (cachedLines) return cachedLines;
      const editorLines = editField !== null ? editor.render(Math.max(20, width - 8)) : [];
      const lines = renderProposalView({
        proposal,
        projectName,
        actionIndex,
        editField,
        editorLines,
        width,
        theme,
        s,
      });
      cachedLines = lines;
      return lines;
    }

    return {
      handleInput,
      render,
      invalidate() {
        cachedLines = undefined;
      },
    };
  });

  return result ?? { action: "rejected", proposal: initialProposal };
}
