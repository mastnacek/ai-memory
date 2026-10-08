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

  // ── Body preview: markdown-lite with a left rule for section headings ──
  lines.push(row(`  ${theme.fg("mdHeading", theme.bold("📄 " + s.bodyLabel))}`));
  const bodyLines = proposal.body.split("\n");
  const previewSlice = bodyLines.slice(0, 8);
  for (const line of previewSlice) {
    if (line.startsWith("### ") || line.startsWith("## ")) {
      const heading = line.replace(/^#+ /, "").trim();
      lines.push(row(`  ${theme.fg("mdHeading", "▐ " + heading)}`));
    } else if (line.trim().startsWith("- ")) {
      lines.push(row(`    ${theme.fg("accent", "•")} ${theme.fg("text", line.trim().slice(2))}`));
    } else if (line.trim().length === 0) {
      lines.push(row(""));
    } else {
      lines.push(row(`    ${theme.fg("dim", line)}`));
    }
  }
  if (bodyLines.length > 8) {
    lines.push(row(`    ${theme.fg("dim", "⋯ " + s.moreLines.replace("{n}", String(bodyLines.length - 8)))}`));
  }
  lines.push(row(""));

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
  } else {
    const scopeToggleLabel = isGlobal
      ? `${s.actionToggleScope} ➔ (📁 ${projectName})`
      : `${s.actionToggleScope} ➔ (🌐 ${s.scopeGlobal})`;

    const actions = [
      s.actionApprove,
      s.actionEditTitle,
      s.actionEditBody,
      s.actionEditTags,
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
