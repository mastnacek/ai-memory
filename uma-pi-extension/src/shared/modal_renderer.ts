import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";
import type { Strings } from "./i18n.js";
import type { MemoryProposal } from "./types.js";

/**
 * The slice of pi's Theme the renderer needs. Structural: the real Theme
 * passed by ctx.ui.custom satisfies it, including bold and the border /
 * selectedBg tokens the older fg-only contract left unused.
 */
type ThemeColor =
  | "accent" | "border" | "borderAccent" | "warning" | "error"
  | "success" | "text" | "muted" | "dim" | "mdHeading" | "toolTitle";

type ThemeRenderer = {
  fg: (color: ThemeColor, text: string) => string;
  bg: (color: "selectedBg", text: string) => string;
  bold: (text: string) => string;
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
  /** The body to DISPLAY — the display-translation cache or the original. */
  bodyText: string;
  /** Status of the display translation, shown as a note under the body. */
  bodyNote?: string;
  /** When set, the body renders as a scrollable full-text window. */
  fullView?: { scrollOffset: number; viewportLines: number };
}

/** Badge emoji per fact type — the glyph is the type's meaning at a glance. */
const TYPE_GLYPHS: Record<string, string> = {
  decision: "⚖️",
  preference: "⭐",
  pattern: "📐",
  skill: "⚡",
  correction: "🔧",
  fact: "📌",
  note: "🗒️",
  task: "🗳️",
  reference: "🔗",
};

export function formatTypeBadge(type: string, theme: ThemeRenderer): string {
  const upper = type.toUpperCase();
  const glyph = TYPE_GLYPHS[type.toLowerCase()] ?? "📄";
  const color: ThemeColor =
    type.toLowerCase() === "correction"
      ? "warning"
      : type.toLowerCase() === "skill" || type.toLowerCase() === "preference"
        ? "success"
        : type.toLowerCase() === "fact" || type.toLowerCase() === "note"
          ? "text"
          : "accent";
  return theme.fg(color, theme.bold(`${glyph}  ${upper}`));
}

function getPromptLabel(field: "title" | "body" | "tags", s: Strings): string {
  if (field === "title") return s.editPromptTitle;
  if (field === "body") return s.editPromptBody;
  return s.editPromptTags;
}

export function renderProposalView(opts: ModalRenderOptions): string[] {
  const { proposal, projectName, actionIndex, editField, editorLines, width, theme, s } = opts;
  const { bodyText, bodyNote, fullView } = opts;
  const lines: string[] = [];
  const renderWidth = Math.max(30, width);
  const inner = renderWidth - 2;
  const isGlobal = proposal.scope.toLowerCase() === "global";

  // ── Rounded frame with the title inlaid in the top border ──────────────
  const titleText = ` ${s.proposalHeader} `;
  const titleW = visibleWidth(titleText);
  const topFill = Math.max(0, inner - titleW - 1);
  lines.push(
    theme.fg("borderAccent", "╭─") +
      theme.fg("borderAccent", theme.bold(titleText)) +
      theme.fg("borderAccent", "─".repeat(topFill) + "╮"),
  );
  lines.push(theme.fg("border", "│" + " ".repeat(inner) + "│"));

  const row = (content: string): string =>
    theme.fg("border", "│ ") + content + " ".repeat(Math.max(0, inner - 2 - visibleWidth(content))) + theme.fg("border", " │");

  // ── Metadata: type badge + scope pill on one line, tags below ─────────
  const scopeDisplay = isGlobal
    ? theme.fg("success", theme.bold("🌐 " + s.scopeGlobal))
    : theme.fg("accent", `📁 ${s.scopeProject} (${projectName})`);
  lines.push(row(`  ${formatTypeBadge(proposal.type, theme)}   ${scopeDisplay}`));

  const tagsDisplay =
    proposal.tags.length > 0
      ? proposal.tags.map((t) => theme.fg("accent", "#" + t)).join(" ")
      : theme.fg("dim", s.noTags);
  lines.push(row(`  ${theme.fg("muted", "🏷️  " + s.tagsLabel + ":")} ${tagsDisplay}`));

  // Chained metadata: supersedes / template / stale / since — compact chips.
  const chain: string[] = [];
  if (proposal.supersedes) {
    chain.push(`${theme.fg("muted", "⤴ " + s.supersedesLabel + ":")} ${theme.fg("warning", proposal.supersedes)}`);
  }
  if (proposal.since) {
    chain.push(`${theme.fg("muted", "⏱ " + s.sinceLabel + ":")} ${theme.fg("dim", proposal.since)}`);
  }
  if (proposal.stale_after) {
    chain.push(`${theme.fg("muted", "⏳ " + s.staleAfterLabel + ":")} ${theme.fg("dim", proposal.stale_after)}`);
  }
  if (proposal.template) {
    chain.push(`${theme.fg("muted", "⌘ " + s.templateLabel + ":")} ${theme.fg("dim", proposal.template)}`);
  }
  for (const chip of chain) lines.push(row(`  ${chip}`));
  lines.push(row(""));

  // ── Title: bold, set off by a heading glyph ────────────────────────────
  lines.push(row(`  ${theme.fg("mdHeading", theme.bold("📌 " + s.titleLabel))}`));
  lines.push(row(`  ${theme.fg("text", theme.bold(proposal.title))}`));
  lines.push(row(""));

  // ── Body: full-text scroll window, or the 8-line preview ──────────────
  const bodyHeading = `  ${theme.fg("mdHeading", theme.bold("📄 " + s.bodyLabel))}`;
  const bodyLines = bodyText.split("\n");
  const renderBodyLine = (line: string, dim: boolean): string => {
    if (line.startsWith("### ") || line.startsWith("## ")) {
      const heading = line.replace(/^#+ /, "").trim();
      return row(`  ${theme.fg("mdHeading", "▐ " + heading)}`);
    }
    if (line.trim().startsWith("- ")) {
      return row(`    ${theme.fg("accent", "•")} ${theme.fg(dim ? "dim" : "text", line.trim().slice(2))}`);
    }
    if (line.trim().length === 0) return row("");
    return row(`    ${theme.fg(dim ? "dim" : "text", line)}`);
  };
  if (fullView) {
    lines.push(row(bodyHeading));
    const { scrollOffset, viewportLines } = fullView;
    for (const line of bodyLines.slice(scrollOffset, scrollOffset + viewportLines)) {
      lines.push(renderBodyLine(line, false));
    }
    const shown = Math.min(viewportLines, Math.max(0, bodyLines.length - scrollOffset));
    const position =
      bodyLines.length > viewportLines
        ? ` ${theme.fg("dim", `[${scrollOffset + 1}–${scrollOffset + shown}/${bodyLines.length}]`)}`
        : "";
    lines.push(row(`  ${theme.fg("dim", "↕ " + s.fullViewHint)}${position}`));
    if (bodyNote) lines.push(row(`  ${theme.fg("dim", bodyNote)}`));
    lines.push(row(""));
  } else {
    lines.push(row(bodyHeading));
    for (const line of bodyLines.slice(0, 8)) {
      lines.push(renderBodyLine(line, true));
    }
    if (bodyLines.length > 8) {
      const hint = s.moreLines.replace("{n}", String(bodyLines.length - 8)) + " · [v] 📜";
      lines.push(row(`    ${theme.fg("dim", hint)}`));
    }
    if (bodyNote) lines.push(row(`  ${theme.fg("dim", bodyNote)}`));
    lines.push(row(""));
  }

  // ── Interactive section ────────────────────────────────────────────────
  if (editField !== null) {
    const promptLabel = getPromptLabel(editField, s);
    lines.push(row(`  ${theme.fg("warning", theme.bold("✎ " + promptLabel))} ${theme.fg("dim", s.editInstruction)}`));
    const boxInner = Math.max(10, inner - 6);
    lines.push(theme.fg("border", "│  ╭" + "─".repeat(boxInner) + "╮ │"));
    for (const el of editorLines) {
      const padded = el + " ".repeat(Math.max(0, boxInner - visibleWidth(el)));
      lines.push(theme.fg("border", "│  │ ") + padded + theme.fg("border", " │ │"));
    }
    lines.push(theme.fg("border", "│  ╰" + "─".repeat(boxInner) + "╯ │"));
    lines.push(row(""));
  } else if (!fullView) {
    const scopeToggleLabel = isGlobal
      ? `${s.actionToggleScope} ➔ (📁 ${projectName})`
      : `${s.actionToggleScope} ➔ (🌐 ${s.scopeGlobal})`;

    const actions = [
      s.actionApprove,
      s.actionEditTitle,
      s.actionEditBody,
      s.actionEditTags,
      s.actionViewFull,
      scopeToggleLabel,
      s.actionReject,
    ];

    lines.push(row(`  ${theme.fg("toolTitle", theme.bold("⚙️  " + s.actionsPrompt))}`));
    for (let i = 0; i < actions.length; i++) {
      const selected = i === actionIndex;
      const content = (selected ? theme.fg("accent", theme.bold("▸ ")) : "  ") + (selected ? theme.fg("text", actions[i]) : theme.fg("dim", actions[i]));
      const pad = " ".repeat(Math.max(0, inner - 2 - visibleWidth(content)));
      // The selected row gets the theme's selected background — a real
      // highlight bar, not just a different glyph.
      lines.push(
        theme.fg("border", "│ ") +
          (selected ? theme.bg("selectedBg", content + pad) : content + pad) +
          theme.fg("border", " │"),
      );
    }
    lines.push(row(""));
  }

  // ── Key-hint footer bar, then the closing border ───────────────────────
  const hints = [
    "↑/↓ " + s.hintSelect,
    "Enter " + s.hintConfirm,
    "Esc " + s.hintCancel,
  ].join("  ·  ");
  const hintLine = `  ${theme.fg("dim", hints)}`;
  lines.push(
    theme.fg("border", "│ ") +
      hintLine +
      " ".repeat(Math.max(0, inner - 1 - visibleWidth(hintLine))) +
      theme.fg("border", "│"),
  );
  lines.push(theme.fg("borderAccent", "╰" + "─".repeat(inner) + "╯"));

  // Width safety: every line passes through truncateToWidth.
  return lines.map((line) => truncateToWidth(line, renderWidth));
}
