import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";
import type { Strings } from "./i18n.js";
import type { MemoryProposal } from "./types.js";

type ThemeRenderer = {
  fg: (color: "accent" | "warning" | "success" | "text" | "muted" | "dim", text: string) => string;
};

export interface ModalRenderOptions {
  proposal: MemoryProposal;
  projectName: string;
  actionIndex: number;
  editField: "title" | "body" | "tags" | null;
  editorLines: string[];
  width: number;
  theme: ThemeRenderer;
  s: Strings;
}

export function formatTypeBadge(type: string, theme: ThemeRenderer): string {
  switch (type.toLowerCase()) {
    case "decision":
      return theme.fg("accent", "⚖️  ROZHODNUTÍ");
    case "preference":
      return theme.fg("warning", "⭐  PREFERENCE");
    case "pattern":
      return theme.fg("accent", "📐  VZOR");
    case "skill":
      return theme.fg("success", "⚡  DOVEDNOST");
    case "correction":
      return theme.fg("warning", "🔧  OPRAVA");
    case "fact":
      return theme.fg("text", "📌  FAKT");
    default:
      return theme.fg("muted", `📝  ${type.toUpperCase()}`);
  }
}

function getPromptLabel(field: "title" | "body" | "tags", s: Strings): string {
  if (field === "title") return s.editPromptTitle;
  if (field === "body") return s.editPromptBody;
  return s.editPromptTags;
}

export function renderProposalView(opts: ModalRenderOptions): string[] {
  const { proposal, projectName, actionIndex, editField, editorLines, width, theme, s } = opts;
  const rawLines: string[] = [];
  const renderWidth = Math.max(30, width);
  const isGlobal = proposal.scope.toLowerCase() === "global";

  // Top Frame Header
  const titleText = ` ${s.proposalHeader} `;
  const titleW = visibleWidth(titleText);
  const topBarLength = Math.max(0, renderWidth - titleW - 4);
  rawLines.push(
    theme.fg("accent", "┌─") +
    theme.fg("accent", titleText) +
    theme.fg("accent", "─".repeat(topBarLength) + "┐")
  );
  rawLines.push(theme.fg("accent", "│"));

  // 1. Metadata: Type & Scope
  const scopeDisplay = isGlobal
    ? theme.fg("success", "🌐 " + s.scopeGlobal)
    : theme.fg("accent", `📁 ${s.scopeProject} (${projectName})`);
  const typeDisplay = formatTypeBadge(proposal.type, theme);

  rawLines.push(`  ${theme.fg("muted", s.typeLabel + ":")} ${typeDisplay}    ${theme.fg("muted", s.scopeLabel + ":")} ${scopeDisplay}`);

  // 2. Metadata: Tags
  const tagsDisplay =
    proposal.tags.length > 0
      ? proposal.tags.map((t) => theme.fg("accent", "#" + t)).join(" ")
      : theme.fg("dim", "(žádné tagy)");
  rawLines.push(`  ${theme.fg("muted", "🏷️  " + s.tagsLabel + ":")} ${tagsDisplay}`);
  if (proposal.supersedes) {
    rawLines.push(
      `  ${theme.fg("muted", "⤴  " + s.supersedesLabel + ":")} ${theme.fg("warning", proposal.supersedes)}`
    );
  }
  if (proposal.template) {
    rawLines.push(
      `  ${theme.fg("muted", "⌘  " + s.templateLabel + ":")} ${theme.fg("dim", proposal.template)}`
    );
  }
  rawLines.push("");

  // 3. Title Box
  rawLines.push(`  ${theme.fg("muted", "📌 Název / Title:")}`);
  rawLines.push(`  ${theme.fg("text", proposal.title)}`);
  rawLines.push(theme.fg("dim", "  " + "─".repeat(Math.max(10, renderWidth - 6))));

  // 4. Body Preview (Markdown)
  rawLines.push(`  ${theme.fg("muted", "📄 Obsah (Markdown preview):")}`);
  const bodyLines = proposal.body.split("\n");
  const previewSlice = bodyLines.slice(0, 8);

  for (const line of previewSlice) {
    if (line.startsWith("### ")) {
      rawLines.push(`    ${theme.fg("accent", "▶ " + line.replace("### ", "").trim())}`);
    } else if (line.startsWith("## ")) {
      rawLines.push(`    ${theme.fg("accent", "■ " + line.replace("## ", "").trim())}`);
    } else if (line.trim().startsWith("- ")) {
      rawLines.push(`      ${theme.fg("text", "• " + line.trim().slice(2))}`);
    } else if (line.trim().length === 0) {
      rawLines.push("");
    } else {
      rawLines.push(`      ${theme.fg("dim", line)}`);
    }
  }

  if (bodyLines.length > 8) {
    rawLines.push(theme.fg("dim", `      ... (+ dalších ${bodyLines.length - 8} řádků)`));
  }
  rawLines.push("");

  // 5. Interactive Section (Editor Mode vs Actions Menu)
  if (editField !== null) {
    const promptLabel = getPromptLabel(editField, s);
    rawLines.push(theme.fg("warning", `  ✎ ${promptLabel} `) + theme.fg("dim", s.editInstruction));
    rawLines.push(theme.fg("warning", "  ┌" + "─".repeat(Math.max(10, renderWidth - 8)) + "┐"));
    for (const el of editorLines) {
      rawLines.push(`  │ ${el}`);
    }
    rawLines.push(theme.fg("warning", "  └" + "─".repeat(Math.max(10, renderWidth - 8)) + "┘"));
    rawLines.push("");
  } else {
    // Actions Menu
    const scopeToggleLabel = isGlobal
      ? `${s.actionToggleScope} ➔ (📁 ${projectName})`
      : `${s.actionToggleScope} ➔ (🌐 Globální)`;

    const actions = [
      s.actionApprove,
      s.actionEditTitle,
      s.actionEditBody,
      s.actionEditTags,
      scopeToggleLabel,
      s.actionReject,
    ];

    rawLines.push(theme.fg("muted", "  ⚙️  Vyberte akci:"));
    for (let i = 0; i < actions.length; i++) {
      const selected = i === actionIndex;
      if (selected) {
        rawLines.push(`  ${theme.fg("accent", "➔ " + actions[i])}`);
      } else {
        rawLines.push(`     ${theme.fg("dim", actions[i])}`);
      }
    }
  }

  // Bottom Border Frame
  rawLines.push(theme.fg("accent", "│"));
  rawLines.push(theme.fg("accent", "└" + "─".repeat(renderWidth - 2) + "┘"));

  // Ensure 100% width safety with truncateToWidth on every single line
  return rawLines.map((line) => truncateToWidth(line, renderWidth));
}
