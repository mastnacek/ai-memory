/**
 * Secret scanning for the proposal path: runs `uma secrets scan --json`
 * BEFORE the approval modal, so a credential-carrying proposal is refused
 * without wasting the operator's review on a write that would only be
 * refused afterwards by the CLI's own gate.
 *
 * VSA note: used by both the write and supersede tool slices, so it lives in
 * shared/ (kernel), not in either slice.
 */

import { runUma, findUmaBinary } from "./client.js";

export interface SecretFinding {
  label: string;
  severity: "block" | "warning";
  preview: string;
}

export interface ScanOutcome {
  blocked: boolean;
  findings: SecretFinding[];
}

/** Outcome used when the scanner itself is unavailable: never blocks on our
 * own failure — the CLI gate after the modal remains the enforcement point. */
const CLEAN: ScanOutcome = { blocked: false, findings: [] };

/**
 * Scans the proposal's text fields. Any failure (no binary, parse error)
 * returns CLEAN: the consent modal must not depend on the scanner being
 * installed, and the CLI write gate runs later regardless.
 */
export async function scanProposal(cwd: string, text: string): Promise<ScanOutcome> {
  if (!text.trim()) return CLEAN;
  try {
    const binPath = findUmaBinary(cwd);
    const result = await runUma(binPath, ["secrets", "scan", "--json", text], cwd);
    if (result.code !== 0) return CLEAN;
    const parsed = JSON.parse(result.stdout) as { blocked: boolean; findings: SecretFinding[] };
    return { blocked: Boolean(parsed.blocked), findings: parsed.findings ?? [] };
  } catch {
    return CLEAN;
  }
}

/** The human-readable refusal listing the findings (masked previews only). */
export function refusalText(outcome: ScanOutcome): string {
  const labels = outcome.findings.map((f) => `${f.label} (${f.preview})`).join(", ");
  return (
    `Refusing to save: the text contains what looks like ${labels}. ` +
    "Memory is re-injected into every future session and may be committed to git — " +
    "never store credentials. Rephrase without the secret " +
    "(name the env var that holds it instead)."
  );
}
