import type { ExtensionState, PluginConfig } from "./types.js";

export function createExtensionState(config: PluginConfig, globalConfigFile: string): ExtensionState {
  return {
    config,
    globalConfigFile,
    unsubscribers: [],
  };
}
