/**
 * Fastbrain recall gate — the S3 compromise.
 *
 * S3 (auto-inject memory facts into turns) was paused by operator choice:
 * uncontrolled injection caused context noise. This hook reopens it *gated*:
 * before each run, a fast judge decides whether the prompt may depend on
 * remembered facts. No trigger → nothing is injected. Trigger → an offline
 * BM25 recall (never a network semantic call per turn) brings the top facts
 * in as one hidden custom message, keeping the system prompt cache-stable.
 *
 * Everything here degrades quiet: the gate must never delay or fail a run.
 * No binary, no key, judge unavailable → the run proceeds with no recall.
 */

import type {
  ExtensionAPI,
  BeforeAgentStartEventResult,
} from "@earendil-works/pi-coding-agent";
import type { ExtensionState } from "../../shared/types.js";
import { runUma, findUmaBinary } from "../../shared/client.js";
import {
  buildRecallMessage,
  type RecallVerdict,
} from "./policy.js";

/** Upper bound on the whole gate: a triage that outlives this is skipped. */
const GATE_TIMEOUT_MS = 8_000;

/** Prompts that are commands, not questions — never gated. */
function isCommand(prompt: string): boolean {
  return prompt.trim().startsWith("/") || prompt.trim().startsWith("!");
}

/** Races a promise against the gate timeout; undefined on timeout. */
function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T | undefined> {
  return Promise.race([
    promise,
    new Promise<undefined>((resolve) => setTimeout(() => resolve(undefined), ms)),
  ]);
}

export function registerFastbrainHook(pi: ExtensionAPI, state: ExtensionState): () => void {
  const unsubscribe = pi.on("before_agent_start", async (event) => {
    // Operator toggle (default off — S3 stays paused unless reopened).
    if (!state.config.recallGate) return undefined;
    if (isCommand(event.prompt)) return undefined;

    const lang = state.config.lang;
    try {
      const binPath = findUmaBinary(process.cwd());
      const judgeArgs =
        state.config.fastbrainJudge === "jev"
          ? ["recall", "check", "--judge", "jev", "--json", event.prompt]
          : ["recall", "check", "--json", event.prompt];

      const output = await withTimeout(runUma(binPath, judgeArgs, process.cwd()), GATE_TIMEOUT_MS);
      if (!output || output.code !== 0) return undefined;

      const verdict = JSON.parse(output.stdout) as RecallVerdict;
      return buildRecallMessage(verdict, lang);
    } catch {
      // The gate is advisory: any failure means "no recall", never "no run".
      return undefined;
    }
  });
  return unsubscribe;
}
