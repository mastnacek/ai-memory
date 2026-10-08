import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import type { ExtensionState } from "../../shared/types.js";
import { registerWriteTool } from "./write.js";
import { registerReadTool } from "./read.js";
import { registerListTool } from "./list.js";
import { registerSearchTool } from "./search.js";
import { registerSupersedeTool } from "./supersede.js";

export function registerTools(pi: ExtensionAPI, state: ExtensionState): void {
  registerWriteTool(pi, state);
  registerReadTool(pi, state);
  registerListTool(pi, state);
  registerSearchTool(pi, state);
  registerSupersedeTool(pi, state);
}
