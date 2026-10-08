import type { ExtensionAPI, ExtensionCommandContext } from "@earendil-works/pi-coding-agent";
import type { ExtensionState } from "../../shared/types.js";
import { runUma, findUmaBinary } from "../../shared/client.js";
import { stringsFor, normalizeLocale } from "../../shared/i18n.js";
import { saveConfig } from "../../shared/config.js";
import { getUmaCompletions } from "./complete.js";

export function registerCommands(pi: ExtensionAPI, state: ExtensionState): void {
  const s = stringsFor(state.config.lang);

  pi.registerCommand("uma", {
    description: s.descUmaCommand,
    getArgumentCompletions: (argumentPrefix: string) =>
      getUmaCompletions(argumentPrefix, state),
    handler: async (argsStr: string, ctx: ExtensionCommandContext) => {
      const liveStrings = stringsFor(state.config.lang);
      const binPath = findUmaBinary(ctx.cwd);
      const parts = argsStr.trim().split(/\s+/).filter(Boolean);
      const isGlobal = parts.includes("--global");
      const cleanParts = parts.filter((p) => p !== "--global");
      const subcommand = cleanParts[0]?.toLowerCase() || "list";

      if (subcommand === "search") {
        const query = cleanParts.slice(1).join(" ");
        if (!query) {
          ctx.ui.notify(liveStrings.searchUsage, "info");
          return;
        }
        const res = await runUma(binPath, ["search", query], ctx.cwd);
        ctx.ui.notify(res.stdout || res.stderr || liveStrings.noFactsFound, "info");
      } else if (subcommand === "list") {
        const scope = cleanParts[1];
        const cmdArgs = ["list"];
        if (scope) cmdArgs.push("--scope", scope);
        const res = await runUma(binPath, cmdArgs, ctx.cwd);
        ctx.ui.notify(res.stdout || liveStrings.noFactsFound, "info");
      } else if (subcommand === "read" && cleanParts[1]) {
        const res = await runUma(binPath, ["read", cleanParts[1]], ctx.cwd);
        ctx.ui.notify(res.stdout || res.stderr, "info");
      } else if (subcommand === "reindex") {
        const res = await runUma(binPath, ["search", "", "--reindex"], ctx.cwd);
        ctx.ui.notify(res.stdout || liveStrings.reindexDone, "info");
      } else if (subcommand === "lang") {
        const target = cleanParts[1];
        if (target === "cs" || target === "en") {
          const loc = normalizeLocale(target);
          saveConfig({ lang: loc }, isGlobal, ctx.cwd, state.globalConfigFile);
          state.config = { ...state.config, lang: loc };
          ctx.ui.notify(`${liveStrings.langUpdated}${loc}`, "info");
        } else {
          ctx.ui.notify(`${liveStrings.langCurrent}${state.config.lang}`, "info");
        }
      } else if (subcommand === "auto-approve") {
        const target = cleanParts[1];
        if (target === "on" || target === "true") {
          saveConfig({ autoApprove: true }, isGlobal, ctx.cwd, state.globalConfigFile);
          state.config = { ...state.config, autoApprove: true };
          ctx.ui.notify(liveStrings.autoApproveEnabled, "info");
        } else if (target === "off" || target === "false") {
          saveConfig({ autoApprove: false }, isGlobal, ctx.cwd, state.globalConfigFile);
          state.config = { ...state.config, autoApprove: false };
          ctx.ui.notify(liveStrings.autoApproveDisabled, "info");
        } else {
          ctx.ui.notify(`${liveStrings.autoApproveCurrent}${state.config.autoApprove ? "ON" : "OFF"}`, "info");
        }
      } else {
        ctx.ui.notify(liveStrings.cmdUsage, "info");
      }
    },
  });
}
