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
  const currentGate = state.config.recallGate;
  const valid_immune = ["off", "warn", "ask", "auto"] as const;
  const currentJudge = state.config.fastbrainJudge;

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

    if (sub === "recall") {
      const subPrefix = tokens[1]?.toLowerCase() || "";
      const options = [
        {
          value: "recall on",
          label: currentGate ? "on ✓" : "on",
          description: currentGate
            ? "Recall gate injects memory on trigger · ● ACTIVE"
            : "Recall gate injects memory on trigger",
        },
        {
          value: "recall off",
          label: !currentGate ? "off ✓" : "off",
          description: !currentGate
            ? "Recall stays explicit (S3 paused) · ● ACTIVE"
            : "Recall stays explicit (S3 paused)",
        },
      ];
      return options.filter((o) => o.value.startsWith(`recall ${subPrefix}`));
    }

    if (sub === "judge") {
      const subPrefix = tokens[1]?.toLowerCase() || "";
      const options = [
        {
          value: "judge jev",
          label: currentJudge === "jev" ? "jev ✓" : "jev",
          description:
            currentJudge === "jev"
              ? "Jev via OpenRouter (semantic) · ● ACTIVE"
              : "Jev via OpenRouter (semantic)",
        },
        {
          value: "judge off",
          label: currentJudge === "off" ? "off ✓" : "off",
          description:
            currentJudge === "off"
              ? "Offline markers (deterministic) · ● ACTIVE"
              : "Offline markers (deterministic)",
        },
      ];
      return options.filter((o) => o.value.startsWith(`judge ${subPrefix}`));
    }

    if (sub === "immune") {
      const subPrefix = tokens[1]?.toLowerCase() || "";
      const current = state.config.immuneMode;
      const descriptions: Record<string, string> = {
        off: "Interceptor disabled",
        warn: "Warnings only, never disturbs",
        ask: "Confirm dialog per warning; decline blocks",
        auto: "Block without asking (contracts) / ask (heuristics)",
      };
      const options = valid_immune.map((m) => ({
        value: `immune ${m}`,
        label: current === m ? `${m} ✓` : m,
        description: current === m ? `${descriptions[m]} · ● ACTIVE` : descriptions[m],
      }));
      return options.filter((o) => o.value.startsWith(`immune ${subPrefix}`));
    }

    if (sub === "list") {
      return [
        { value: "list", label: "project", description: "List project memories" },
        { value: "list global", label: "global", description: "List global user memories" },
      ];
    }

    if (sub === "timeline") {
      const subPrefix = tokens[1]?.toLowerCase() || "";
      return [
        {
          value: "timeline --all",
          label: "--all",
          description: "Include facts that were never superseded",
        },
        {
          value: "timeline --id ",
          label: "--id <ULID>",
          description: "Show only the chain containing this fact",
        },
      ].filter((o) => o.value.startsWith(`timeline ${subPrefix}`));
    }

    if (sub === "export") {
      const subPrefix = tokens[1]?.toLowerCase() || "";
      return [
        {
          value: "export --out ",
          label: "--out <directory>",
          description: "Write an OKF v0.2 bundle (one Markdown document per fact)",
        },
        {
          value: "export --out . --include-deprecated",
          label: "--include-deprecated",
          description: "Also export superseded revisions",
        },
      ].filter((o) => o.value.startsWith(`export ${subPrefix}`));
    }

    if (sub === "doctor") {
      return [
        {
          value: "doctor --json",
          label: "--json",
          description: "Emit the report as JSON",
        },
        {
          value: "doctor --strict",
          label: "--strict",
          description: "Exit non-zero when a check fails",
        },
      ].filter((o) => o.value.startsWith("doctor"));
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
      value: "timeline",
      label: "timeline",
      description: "Show how facts evolved (supersession chains)",
    },
    {
      value: "export ",
      label: "export",
      description: "Write memory as a portable OKF bundle",
    },
    {
      value: "doctor",
      label: "doctor",
      description: "Read-only health report on the store and index",
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
    {
      value: "recall ",
      label: `recall (${currentGate ? "on" : "off"})`,
      description: `Toggle the fastbrain recall gate (current: ${currentGate ? "on" : "off"})`,
    },
    {
      value: "judge ",
      label: `judge (${currentJudge})`,
      description: `Recall judge transport (current: ${currentJudge})`,
    },
    {
      value: "immune ",
      label: `immune (${state.config.immuneMode})`,
      description: "Immune interceptor mode (off | warn | ask | auto)",
    },
  ];

  return subcommands.filter((item) => item.value.startsWith(prefix));
}
