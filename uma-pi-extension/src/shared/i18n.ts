export const LOCALES = ["en", "cs"] as const;
export type Locale = (typeof LOCALES)[number];

export interface Strings {
  proposalHeader: string;
  typeLabel: string;
  scopeLabel: string;
  tagsLabel: string;
  bodyLabel: string;
  actionApprove: string;
  actionEditTitle: string;
  actionEditBody: string;
  actionEditTags: string;
  actionToggleScope: string;
  actionReject: string;
  editPromptTitle: string;
  editPromptBody: string;
  editPromptTags: string;
  editInstruction: string;
  savedNotification: string;
  rejectedNotification: string;
  noFactsFound: string;
  langUpdated: string;
  cmdUsage: string;
  searchUsage: string;
  reindexDone: string;
  langCurrent: string;
  autoApproveEnabled: string;
  autoApproveDisabled: string;
  autoApproveCurrent: string;
  descUmaCommand: string;
  scopeGlobal: string;
  scopeProject: string;
}

const STRINGS: Record<Locale, Strings> = {
  en: {
    proposalHeader: "🧠 UMA • Memory Proposal",
    typeLabel: "Type",
    scopeLabel: "Scope",
    tagsLabel: "Tags",
    bodyLabel: "Content",
    actionApprove: "[Enter]  ✅  Approve & Save to Memory",
    actionEditTitle: "[e]      ✏️   Edit Title",
    actionEditBody: "[b]      📄  Edit Content (Markdown)",
    actionEditTags: "[t]      🏷️   Edit Tags",
    actionToggleScope: "[s]      🔄  Toggle Scope",
    actionReject: "[Esc]    ❌  Reject / Discard",
    editPromptTitle: "Edit Title:",
    editPromptBody: "Edit Content (Markdown):",
    editPromptTags: "Edit Tags (comma-separated):",
    editInstruction: "[Enter: Save change • Esc: Cancel]",
    savedNotification: "Memory saved to UMA",
    rejectedNotification: "Memory proposal rejected",
    noFactsFound: "No facts found.",
    langUpdated: "Language updated to: ",
    cmdUsage: "Usage: /uma [search <query> | list [global] | read <id> | lang [cs|en] [--global] | auto-approve [on|off] [--global]]",
    searchUsage: "Usage: /uma search <query>",
    reindexDone: "Centralized index rebuilt successfully.",
    langCurrent: "Current language: ",
    autoApproveEnabled: "Auto-approval enabled (modal review skipped)",
    autoApproveDisabled: "Auto-approval disabled (modal review active)",
    autoApproveCurrent: "Auto-approve state: ",
    descUmaCommand: "Universal Memory Architecture (UMA) manager",
    scopeGlobal: "Global (all projects)",
    scopeProject: "Project",
  },
  cs: {
    proposalHeader: "🧠 UMA • Návrh zápisu do paměti",
    typeLabel: "Typ",
    scopeLabel: "Rozsah",
    tagsLabel: "Tagy",
    bodyLabel: "Obsah",
    actionApprove: "[Enter]  ✅  Schválit a uložit do paměti",
    actionEditTitle: "[e]      ✏️   Upravit název",
    actionEditBody: "[b]      📄  Upravit obsah (Markdown)",
    actionEditTags: "[t]      🏷️   Upravit tagy",
    actionToggleScope: "[s]      🔄  Přepnout rozsah",
    actionReject: "[Esc]    ❌  Zamítnout a zahodit",
    editPromptTitle: "Upravit název:",
    editPromptBody: "Upravit obsah (Markdown):",
    editPromptTags: "Upravit tagy (oddělené čárkou):",
    editInstruction: "[Enter: Uložit změnu • Esc: Zpět]",
    savedNotification: "Záznam byl uložen do UMA",
    rejectedNotification: "Návrh paměti byl zamítnut",
    noFactsFound: "Nebyly nalezeny žádné záznamy.",
    langUpdated: "Jazyk rozhraní nastaven na: ",
    cmdUsage: "Použití: /uma [search <dotaz> | list [global] | read <id> | lang [cs|en] [--global] | auto-approve [on|off] [--global]]",
    searchUsage: "Použití: /uma search <dotaz>",
    reindexDone: "Centralizovaný index byl úspěšně přebudován.",
    langCurrent: "Aktuální jazyk: ",
    autoApproveEnabled: "Automatické schvalování zapnuto (modální okno se nezobrazuje)",
    autoApproveDisabled: "Automatické schvalování vypnuto (modální okno je aktivní)",
    autoApproveCurrent: "Stav automatického schvalování: ",
    descUmaCommand: "Správa paměťového systému UMA",
    scopeGlobal: "Globální (všechny projekty)",
    scopeProject: "Projekt",
  },
};

export function normalizeLocale(raw: string | undefined): Locale {
  const token = (raw ?? "").trim().toLowerCase();
  if (token === "cs" || token === "cz" || token === "cze") return "cs";
  return "en";
}

export function stringsFor(locale: string | undefined): Strings {
  return STRINGS[normalizeLocale(locale)];
}
