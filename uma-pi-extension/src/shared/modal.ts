import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { Editor, type EditorTheme, Key, matchesKey } from "@earendil-works/pi-tui";
import * as path from "node:path";
import type { MemoryProposal, ProposalResult } from "./types.js";
import { stringsFor } from "./i18n.js";
import { renderProposalView } from "./modal_renderer.js";
import { DisplayTranslation } from "../slices/translate/index.js";

type EditField = "title" | "body" | "tags" | null;

function resolveProjectName(cwd: string): string {
  const base = path.basename(cwd);
  return base || "project";
}

/// Maps the proposal's scope for display and editing in the modal.
///
/// The generic default "project" (the tool's fallback) and an empty scope
/// become the current project name; "global" stays global; an explicit scope
/// ("mozek_rust", a dead project imported from a session) passes through
/// untouched — the reviewer can still toggle it with the scope action.
export function normalizeScope(initial: string, projectName: string): string {
  const lowered = initial.trim().toLowerCase();
  if (lowered === "global") return "global";
  if (lowered === "" || lowered === "project") return projectName;
  return initial;
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
  const normalizedInitialScope = normalizeScope(initialProposal.scope, projectName);

  const result = await ctx.ui.custom<ProposalResult | null>((tui, theme, _kb, done) => {
    // Fields the reviewer does not edit (supersedes, template, stale_after,
    // since) must survive the round-trip: dropping them silently changed
    // what would be stored — an approved proposal is the reviewed contract,
    // so everything the tool proposed has to come back in the result.
    const proposal: MemoryProposal = {
      ...initialProposal,
      title: initialProposal.title,
      body: initialProposal.body,
      type: initialProposal.type,
      scope: normalizedInitialScope,
      tags: [...initialProposal.tags],
    };

    let actionIndex = 0;
    let editField: EditField = null;
    let cachedLines: string[] | undefined;

    // Display translation + full-text scroll state; `proposal` must keep the
    // ORIGINAL body because what the modal returns on approval is what gets
    // stored (see slices/translate/README.md).
    let showTranslation = lang === "cs";
    let fullView = false;
    let scrollOffset = 0;
    const VIEWPORT = 16;

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

    const displayTranslation = new DisplayTranslation(ctx, refresh);
    if (showTranslation) displayTranslation.start(proposal.body);

    /** The body the modal displays — translated cache or the original. */
    function displayBody(): string {
      return displayTranslation.bodyFor(proposal.body, showTranslation);
    }

    function fullBodyLines(): string[] {
      return displayBody().split("\n");
    }

    function clampScroll(): void {
      const max = Math.max(0, fullBodyLines().length - VIEWPORT);
      scrollOffset = Math.min(Math.max(0, scrollOffset), max);
    }

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

      // ── Full-text scroll view: arrows scroll, Esc returns to the menu ──
      if (fullView) {
        if (matchesKey(data, Key.up)) {
          scrollOffset = Math.max(0, scrollOffset - 1);
          refresh();
          return;
        }
        if (matchesKey(data, Key.down)) {
          scrollOffset = Math.min(
            Math.max(0, fullBodyLines().length - VIEWPORT),
            scrollOffset + 1,
          );
          refresh();
          return;
        }
        if (data === "c" || data === "C") {
          showTranslation = !showTranslation;
          clampScroll();
          refresh();
          return;
        }
        if (matchesKey(data, Key.escape) || matchesKey(data, Key.enter)) {
          fullView = false;
          refresh();
          return;
        }
        return;
      }

      const actionsCount = 7;

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
      if (data === "v" || data === "V") {
        fullView = true;
        scrollOffset = 0;
        refresh();
        return;
      }
      if (data === "c" || data === "C") {
        showTranslation = !showTranslation;
        refresh();
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
            fullView = true;
            scrollOffset = 0;
            refresh();
            break;
          case 5:
            toggleScope();
            break;
          case 6:
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
      // The note explains what the reviewer is looking at: pending / shown /
      // failed translation — never silently swap the language.
      const bodyNote =
        editField === null
          ? displayTranslation.pending
            ? s.translatingNote
            : showTranslation && !displayTranslation.failed
              ? s.translatedNote
              : displayTranslation.failed
                ? s.translationFailedNote
                : undefined
          : undefined;
      const lines = renderProposalView({
        proposal,
        projectName,
        actionIndex,
        editField,
        editorLines,
        width,
        theme,
        s,
        bodyText: displayBody(),
        bodyNote,
        fullView: fullView ? { scrollOffset, viewportLines: VIEWPORT } : undefined,
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
