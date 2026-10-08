export interface MemoryProposal {
  title: string;
  body: string;
  type: string;
  scope: string;
  tags: string[];
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
