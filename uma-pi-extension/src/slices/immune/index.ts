/**
 * Immune interceptor — warn-mode (the advisory half of Proposal 01).
 *
 * Before a file-mutating tool call lands, the active memory rules and the
 * file's pain score are checked and any hazard is surfaced to the operator
 * as a warning. This mode **never blocks**: blocking on a probabilistic
 * verdict would invert the consent model; block-mode is reserved for the
 * deterministic contract-backed rules of Proposal 03.
 *
 * Pure decision logic lives here (testable without pi); the thin event
 * subscription sits in hooks/immune_interceptor.ts and is wired — tracked —
 * in the composition root, per the hook-purity rule.
 */

/** Verdict of `uma risk pain <path> --json`. */
export interface PainVerdict {
  score: number;
  band: "low" | "medium" | "critical";
}

/** One L1 rule: what the interceptor checks proposed edits against. */
export interface RuleL1 {
  id: string;
  title: string;
  fact_type: string;
  body: string;
}

/** Everything the interceptor needs from the outside, injectable for tests. */
export interface ImmuneDeps {
  painScore(cwd: string, path: string): Promise<PainVerdict | undefined>;
  rules(cwd: string): Promise<RuleL1[]>;
}

/** Tokenizes text into lowercase words long enough to carry meaning. */
function tokenize(text: string): string[] {
  return text
    .toLowerCase()
    .split(/[^\p{L}\p{N}]+/u)
    .filter((token) => token.length >= 4);
}

/** Jaccard overlap of two token bags. */
function overlap(a: string[], b: string[]): number {
  if (a.length === 0 || b.length === 0) return 0;
  const setA = new Set(a);
  const setB = new Set(b);
  let intersection = 0;
  for (const token of setA) {
    if (setB.has(token)) intersection += 1;
  }
  return intersection / (setA.size + setB.size - intersection);
}

/**
 * Token overlap above which an edit "touches" a rule — close enough that the
 * operator should see the rule before the code lands. Deliberately low: this
 * mode warns, and a warning that never fires is worthless.
 */
const RULE_OVERLAP_THRESHOLD = 0.22;

/**
 * Pure decision: which warnings does this edit deserve?
 * Returns human-readable warnings; empty means "nothing to say". Never
 * produces a block — the return value is fed to a notification, nothing else.
 */
export function assessEdit(
  pain: PainVerdict | undefined,
  rules: RuleL1[],
  addedText: string,
): string[] {
  const warnings: string[] = [];

  if (pain?.band === "medium") {
    warnings.push(`[UMA Risk] Pain score ${pain.score}/100 for this file — run the affected tests after editing.`);
  }
  if (pain?.band === "critical") {
    warnings.push(`[UMA Risk] Pain score ${pain.score}/100 — test-first: propose the failing test before changing this file.`);
  }

  const addedTokens = tokenize(addedText);
  if (addedTokens.length >= 5) {
    for (const rule of rules) {
      const ruleTokens = tokenize(`${rule.title} ${rule.body}`);
      if (overlap(addedTokens, ruleTokens) >= RULE_OVERLAP_THRESHOLD) {
        warnings.push(
          `[UMA Rule] This edit may touch active rule ${rule.id} "${rule.title}" — verify compliance before saving.`,
        );
        break; // one rule warning per edit: enough to look, not noise
      }
    }
  }

  return warnings;
}

/** Extracts the path and the added text from a write/edit tool input. */
export function extractEdit(toolName: string, input: unknown): { path: string; added: string } | undefined {
  if (typeof input !== "object" || input === null) return undefined;
  const record = input as Record<string, unknown>;
  const path = record.path;
  if (typeof path !== "string" || !path) return undefined;

  if (toolName === "write" && typeof record.content === "string") {
    return { path, added: record.content };
  }
  if (toolName === "edit" && Array.isArray(record.edits)) {
    const added = record.edits
      .map((edit) => (typeof edit === "object" && edit !== null ? (edit as Record<string, unknown>).newText : undefined))
      .filter((text): text is string => typeof text === "string")
      .join("\n");
    return { path, added };
  }
  return undefined;
}
