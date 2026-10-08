/**
 * Fastbrain policy: the pure decision half of the recall gate.
 *
 * Deliberately import-free so `node --test` (strip-types) can exercise the
 * policy directly; the transport half lives in index.ts.
 */

/** Verdict of `uma recall check --json`. */
export interface RecallVerdict {
  search: boolean;
  judged_by: string;
  fact_types: string[];
  recalled: number;
  facts: RecallFact[];
  note: string | null;
}

/** The CLI serializes the Scope enum in serde's default external form. */
export type ScopeWire = { Project: string } | "Global";

export interface RecallFact {
  id: string;
  title: string;
  fact_type: string;
  scope: ScopeWire;
  score: number;
  snippet: string;
  tags: string[];
}

/** The injected message's shape (BeforeAgentStartEventResult["message"]). */
export interface RecallMessage {
  customType: string;
  content: string;
  display: boolean;
  details: { judge: string; count: number };
}

/** Renders the scope for display: "Global" becomes "global", the tagged
 * variant becomes "project:<name>" — interpolating the raw object was the
 * [object Object] seen in the first live recall injection. */
export function formatScope(scope: ScopeWire): string {
  return typeof scope === "string" ? "global" : `project:${scope.Project}`;
}

/**
 * Pure decision: what should the hook do with a verdict?
 * No trigger or no facts → nothing is injected.
 */
export function buildRecallMessage(
  verdict: RecallVerdict,
  lang: "cs" | "en",
): { message: RecallMessage } | undefined {
  if (!verdict.search || verdict.facts.length === 0) return undefined;
  const lines = verdict.facts.map(
    (fact) =>
      `- [${fact.fact_type}] ${fact.title} (${formatScope(fact.scope)}): ${fact.snippet}`,
  );
  const header =
    lang === "cs"
      ? "Paměť k tomuto úkolu (použij, kde je relevantní):"
      : "Memory relevant to this task (use where applicable):";
  const content = `${header}\n${lines.join("\n")}`;
  return {
    message: {
      customType: "uma-recall",
      content,
      display: false,
      details: { judge: verdict.judged_by, count: verdict.facts.length },
    },
  };
}

/** Truncates a prompt for the one-line decision report. */
function elide(text: string, width: number): string {
  const single = text.replace(/\s+/g, " ").trim();
  if (single.length <= width) return single;
  return `${single.slice(0, width - 1)}…`;
}

/**
 * Formats the gate's decision as the console report: what was asked, what
 * the judge answered, and what happens next. One compact block per turn —
 * observability the operator asked for, without becoming the context noise
 * the gate exists to prevent (this goes to the UI, never to the model).
 */
export function formatGateDecision(
  prompt: string,
  verdict: RecallVerdict,
  lang: "cs" | "en",
): string {
  const asked =
    lang === "cs"
      ? `Soudce ${verdict.judged_by} · „${elide(prompt, 60)}"`
      : `Judge ${verdict.judged_by} · "${elide(prompt, 60)}"`;
  const types = verdict.fact_types.join(", ");

  if (!verdict.search) {
    return lang === "cs"
      ? `🧠 ${asked} → žádný trigger → paměť zůstává zavřená`
      : `🧠 ${asked} → no trigger → memory stays closed`;
  }

  const next =
    verdict.recalled > 0
      ? lang === "cs"
        ? `injektuju ${verdict.recalled} fakta (${elide(types, 40)})`
        : `injecting ${verdict.recalled} fact(s) (${elide(types, 40)})`
      : lang === "cs"
        ? "trigger, ale nic relevatního nenalezeno → nic se neinjektuje"
        : "trigger, nothing relevant found → injecting nothing";

  const degraded = verdict.note ? ` ⚠ ${verdict.note}` : "";
  return `🧠 ${asked} → trigger (${types || "general"}) → ${next}${degraded}`;
}
