export interface MemoryProposal {
  title: string;
  body: string;
  type: string;
  scope: string;
  tags: string[];
  /** ULID of the fact this proposal replaces, when superseding. */
  supersedes?: string;
  /** Invocation template for `skill` facts. Displayed for review; never executed. */
  template?: string;
  /** ISO 8601 date/timestamp after which the claim needs re-verification. */
  stale_after?: string;
  /** ISO 8601 date/timestamp when the claim started to hold (imports). */
  since?: string;
}

export interface ProposalResult {
  action: "approved" | "rejected";
  proposal: MemoryProposal;
}

export interface PluginConfig {
  lang: "cs" | "en";
  autoApprove: boolean;
  /** Fastbrain recall gate (S3 compromise): OFF — recall stays explicit. */
  recallGate: boolean;
  /** Immune interceptor mode: off | warn (default) | ask | auto. */
  immuneMode: "off" | "warn" | "ask" | "auto";
  /** Recall judge transport: "off" (markers) or "jev" (Jev via OpenRouter). */
  fastbrainJudge: "off" | "jev";
}

export interface ExtensionState {
  config: PluginConfig;
  globalConfigFile: string;
  unsubscribers: Array<() => void>;
}
