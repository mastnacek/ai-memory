/**
 * Immune interceptor hook — thin subscription over the pure warn-mode
 * decision in slices/immune. The composition root owns the registration and
 * its tracked unsubscribe; this file only adapts pi's tool_call event to the
 * pure assessment and to UI notifications.
 *
 * Warn-only: this hook NEVER returns a block. It never throws into pi —
 * every failure degrades to silence, because an interceptor that breaks the
 * agent's work is worse than no interceptor.
 */

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import type { ExtensionState } from "../shared/types.js";
import { runUma, findUmaBinary } from "../shared/client.js";
import { assessEdit, extractEdit, type PainVerdict, type RuleL1 } from "../slices/immune/index.js";

/** Upper bound per check: a warning that costs 5 seconds is worse than none. */
const CHECK_TIMEOUT_MS = 5_000;

/** The L1 rules cache: decisions and patterns only, refreshed at most every 10 minutes. */
const RULES_CACHE_TTL_MS = 10 * 60 * 1_000;

interface CachedRules {
  rules: RuleL1[];
  fetchedAt: number;
}

let rulesCache: CachedRules | undefined;

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T | undefined> {
  return Promise.race([
    promise,
    new Promise<undefined>((resolve) => setTimeout(() => resolve(undefined), ms)),
  ]);
}

async function fetchPain(cwd: string, path: string): Promise<PainVerdict | undefined> {
  const binPath = findUmaBinary(cwd);
  const output = await withTimeout(runUma(binPath, ["risk", "pain", path, "--json"], cwd), CHECK_TIMEOUT_MS);
  if (!output || output.code !== 0) return undefined;
  try {
    return JSON.parse(output.stdout) as PainVerdict;
  } catch {
    return undefined;
  }
}

async function fetchRules(cwd: string): Promise<RuleL1[]> {
  const now = Date.now();
  if (rulesCache && now - rulesCache.fetchedAt < RULES_CACHE_TTL_MS) {
    return rulesCache.rules;
  }
  const binPath = findUmaBinary(cwd);
  const output = await withTimeout(
    runUma(binPath, ["list", "--json", "--type", "decision", "--type", "pattern"], cwd),
    CHECK_TIMEOUT_MS,
  );
  // One --type flag is accepted by clap but only the last wins; fetch both
  // scopes separately and merge.
  const decisions = output?.code === 0 ? output.stdout : undefined;
  const patterns = await withTimeout(
    runUma(binPath, ["list", "--json", "--type", "pattern"], cwd),
    CHECK_TIMEOUT_MS,
  );
  const rules: RuleL1[] = [];
  for (const raw of [decisions, patterns?.code === 0 ? patterns.stdout : undefined]) {
    if (!raw) continue;
    try {
      const parsed = JSON.parse(raw) as RuleL1[];
      rules.push(...parsed);
    } catch {
      // skip an unparsable response; the cache stays smaller, not wrong
    }
  }
  rulesCache = { rules, fetchedAt: now };
  return rules;
}

/** Public reset for tests: caches must never leak between test cases. */
export function resetImmuneCache(): void {
  rulesCache = undefined;
}

export function registerImmuneInterceptor(pi: ExtensionAPI, state: ExtensionState): () => void {
  const unsubscribe = pi.on("tool_call", async (event, ctx) => {
    // Warn-only v1: only file-mutating code tools are checked at all.
    const edit = extractEdit(event.toolName, event.input);
    if (!edit) return undefined;

    try {
      const [pain, rules] = await Promise.all([
        fetchPain(ctx.cwd, edit.path),
        fetchRules(ctx.cwd),
      ]);
      const warnings = assessEdit(pain, rules, edit.added);
      if (warnings.length > 0) {
        // UI-only: the agent and the transcript never see these; the
        // operator decides what to do with them. NEVER a block.
        ctx.ui.notify(warnings.join("\n"), "warning");
      }
    } catch {
      // Advisory half of the consent model: silence is the safe fallback.
    }
    return undefined;
  });
  return unsubscribe;
}