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

export interface RecallFact {
  id: string;
  title: string;
  fact_type: string;
  scope: string;
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
    (fact) => `- [${fact.fact_type}] ${fact.title} (${fact.scope}): ${fact.snippet}`,
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
