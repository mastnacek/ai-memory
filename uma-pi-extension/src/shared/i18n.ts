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
  timelineUsage: string;
  exportUsage: string;
  reindexDone: string;
  langCurrent: string;
  autoApproveEnabled: string;
  autoApproveDisabled: string;
  autoApproveCurrent: string;
  recallGateEnabled: string;
  recallGateDisabled: string;
  recallGateCurrent: string;
  recallJudgeLabel: string;
  immuneModeEnabled: string;
  immuneModeCurrent: string;
  immuneModeLabel: string;
  descUmaCommand: string;
  scopeGlobal: string;
  scopeProject: string;
  supersedesLabel: string;
  templateLabel: string;
  staleAfterLabel: string;
  sinceLabel: string;
  noTags: string;
  titleLabel: string;
  moreLines: string;
  actionsPrompt: string;
  hintSelect: string;
  hintConfirm: string;
  hintCancel: string;
  actionViewFull: string;
  fullViewHint: string;
  translatingNote: string;
  translatedNote: string;
  translationFailedNote: string;
  displayTranslatedNote: string;
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
    actionViewFull: "[v]      📜  Full text (scrollable)",
    fullViewHint: "↑/↓ scroll · c original/Czech · Esc back",
    translatingNote: "🌐 translating for display… (the original is stored)",
    translatedNote: "🌐 display translation — the ORIGINAL is saved (c: original)",
    translationFailedNote: "🌐 translation unavailable — showing the original",
    displayTranslatedNote: "🌐 display translation — memory and CLI output unchanged",
    editPromptTitle: "Edit Title:",
    editPromptBody: "Edit Content (Markdown):",
    editPromptTags: "Edit Tags (comma-separated):",
    editInstruction: "[Enter: Save change • Esc: Cancel]",
    savedNotification: "Memory saved to UMA",
    rejectedNotification: "Memory proposal rejected",
    noFactsFound: "No facts found.",
    langUpdated: "Language updated to: ",
    cmdUsage: "Usage: /uma [search <query> | list [global] | read <id> | timeline [--id <ULID>] [--all] | export --out <dir> | doctor | lang [cs|en] | auto-approve [on|off] | recall [on|off] | judge [jev|off] — toggles accept --global]",
    searchUsage: "Usage: /uma search <query>",
    timelineUsage: "Usage: /uma timeline [--id <ULID>] [--all]",
    exportUsage: "Usage: /uma export --out <directory>",
    reindexDone: "Centralized index rebuilt successfully.",
    langCurrent: "Current language: ",
    autoApproveEnabled: "Auto-approval enabled (modal review skipped)",
    autoApproveDisabled: "Auto-approval disabled (modal review active)",
    autoApproveCurrent: "Auto-approve state: ",
    recallGateEnabled: "Fastbrain recall gate ON (memory injected only when the judge triggers)",
    recallGateDisabled: "Fastbrain recall gate OFF (recall stays explicit)",
    recallGateCurrent: "Recall gate state: ",
    recallJudgeLabel: "Judge",
    immuneModeEnabled: "Immune interceptor mode updated: ",
    immuneModeCurrent: "Immune interceptor mode: ",
    immuneModeLabel: "Immune",
    descUmaCommand: "Universal Memory Architecture (UMA) manager",
    scopeGlobal: "Global (all projects)",
    scopeProject: "Project",
    supersedesLabel: "Supersedes",
    templateLabel: "Template (not executed)",
    staleAfterLabel: "Valid until (re-verify after)",
    sinceLabel: "Valid from",
    noTags: "(no tags)",
    titleLabel: "Title",
    moreLines: "+{n} more lines",
    actionsPrompt: "Choose an action:",
    hintSelect: "select",
    hintConfirm: "confirm",
    hintCancel: "cancel",
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
    actionViewFull: "[v]      📜  Celý text (s rolováním)",
    fullViewHint: "↑/↓ rolovat · c originál/česky · Esc zpět",
    translatingNote: "🌐 překládám pro zobrazení… (ukládá se originál)",
    translatedNote: "🌐 zobrazený překlad — ukládá se ORIGINÁL (c: originál)",
    translationFailedNote: "🌐 překlad nedostupný — zobrazuje se originál",
    displayTranslatedNote: "🌐 zobrazený překlad — pamět i výstup CLI se nemění",
    editPromptTitle: "Upravit název:",
    editPromptBody: "Upravit obsah (Markdown):",
    editPromptTags: "Upravit tagy (oddělené čárkou):",
    editInstruction: "[Enter: Uložit změnu • Esc: Zpět]",
    savedNotification: "Záznam byl uložen do UMA",
    rejectedNotification: "Návrh paměti byl zamítnut",
    noFactsFound: "Nebyly nalezeny žádné záznamy.",
    langUpdated: "Jazyk rozhraní nastaven na: ",
    cmdUsage: "Použití: /uma [search <dotaz> | list [global] | read <id> | timeline [--id <ULID>] [--all] | export --out <složka> | doctor | lang [cs|en] | auto-approve [on|off] | recall [on|off] | judge [jev|off] — přepínače akceptují --global]",
    searchUsage: "Použití: /uma search <dotaz>",
    timelineUsage: "Použití: /uma timeline [--id <ULID>] [--all]",
    exportUsage: "Použití: /uma export --out <složka>",
    reindexDone: "Centralizovaný index byl úspěšně přebudován.",
    langCurrent: "Aktuální jazyk: ",
    autoApproveEnabled: "Automatické schvalování zapnuto (modální okno se nezobrazuje)",
    autoApproveDisabled: "Automatické schvalování vypnuto (modální okno je aktivní)",
    autoApproveCurrent: "Stav automatického schvalování: ",
    recallGateEnabled: "Recallová brána ZAPNUTÁ (paměť se injektuje jen při triggeru)",
    recallGateDisabled: "Recallová brána VYPNUTÁ (recall zůstává explicitní)",
    recallGateCurrent: "Stav recallové brány: ",
    recallJudgeLabel: "Soudce",
    immuneModeEnabled: "Režim imunního interceptoru nastaven: ",
    immuneModeCurrent: "Režim imunního interceptoru: ",
    immuneModeLabel: "Imunita",
    descUmaCommand: "Správa paměťového systému UMA",
    scopeGlobal: "Globální (všechny projekty)",
    scopeProject: "Projekt",
    supersedesLabel: "Nahrazuje",
    templateLabel: "Šablona (nespouští se)",
    staleAfterLabel: "Platí do (poté znovu ověřit)",
    sinceLabel: "Platí od",
    noTags: "(žádné tagy)",
    titleLabel: "Název",
    moreLines: "+{n} řádků",
    actionsPrompt: "Vyberte akci:",
    hintSelect: "vybrat",
    hintConfirm: "potvrdit",
    hintCancel: "zrušit",
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
