import type { AutocompleteItem } from "@earendil-works/pi-tui";
import type { ExtensionState } from "../../shared/types.js";

export function getUmaCompletions(
  argumentPrefix: string,
  state: ExtensionState
): AutocompleteItem[] {
  const trimmed = argumentPrefix.trimStart();
  const trailingSpace = argumentPrefix.endsWith(" ");
  const tokens = trimmed.split(/\s+/).filter(Boolean);

  const currentLang = state.config.lang;
  const currentAuto = state.config.autoApprove;

  // Level 2: Subcommand parameters
  if (tokens.length > 1 || (trailingSpace && tokens.length === 1)) {
    const sub = tokens[0]?.toLowerCase();

    if (sub === "lang") {
      const subPrefix = tokens[1]?.toLowerCase() || "";
      const options = [
        {
          value: "lang cs",
          label: currentLang === "cs" ? "cs ✓" : "cs",
          description: currentLang === "cs" ? "Čeština · ● AKTIVNÍ" : "Čeština",
        },
        {
          value: "lang en",
          label: currentLang === "en" ? "en ✓" : "en",
          description: currentLang === "en" ? "English · ● ACTIVE" : "English",
        },
      ];
      return options.filter((o) => o.value.startsWith(`lang ${subPrefix}`));
    }

    if (sub === "auto-approve") {
      const subPrefix = tokens[1]?.toLowerCase() || "";
      const options = [
        {
          value: "auto-approve on",
          label: currentAuto ? "on ✓" : "on",
          description: currentAuto ? "Auto-approve without modal · ● ACTIVE" : "Auto-approve without modal",
        },
        {
          value: "auto-approve off",
          label: !currentAuto ? "off ✓" : "off",
          description: !currentAuto ? "Show review modal · ● ACTIVE" : "Show review modal",
        },
      ];
      return options.filter((o) => o.value.startsWith(`auto-approve ${subPrefix}`));
    }

    if (sub === "list") {
      return [
        { value: "list", label: "project", description: "List project memories" },
        { value: "list global", label: "global", description: "List global user memories" },
      ];
    }

    return [];
  }

  // Level 1: Top-level subcommands
  const prefix = tokens[0]?.toLowerCase() || "";
  const subcommands: AutocompleteItem[] = [
    {
      value: "search ",
      label: "search",
      description: "Search memory facts (BM25 keyword search)",
    },
    {
      value: "list",
      label: "list",
      description: "List project memories",
    },
    {
      value: "list global",
      label: "list global",
      description: "List global user memories",
    },
    {
      value: "read ",
      label: "read",
      description: "Read memory fact details by ID",
    },
    {
      value: "reindex",
      label: "reindex",
      description: "Rebuild centralized SQLite index from markdown files",
    },
    {
      value: "lang ",
      label: `lang (${currentLang})`,
      description: `Switch UI language (current: ${currentLang})`,
    },
    {
      value: "auto-approve ",
      label: `auto-approve (${currentAuto ? "on" : "off"})`,
      description: `Toggle review modal (current: ${currentAuto ? "on" : "off"})`,
    },
  ];

  return subcommands.filter((item) => item.value.startsWith(prefix));
}
