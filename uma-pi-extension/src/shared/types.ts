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
}

export interface ProposalResult {
  action: "approved" | "rejected";
  proposal: MemoryProposal;
}

export interface PluginConfig {
  lang: "cs" | "en";
  autoApprove: boolean;
}

export interface ExtensionState {
  config: PluginConfig;
  globalConfigFile: string;
  unsubscribers: Array<() => void>;
}
