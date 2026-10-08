import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import type { PluginConfig } from "./types.js";

const DEFAULT_CONFIG: PluginConfig = {
  lang: "cs",
  autoApprove: false,
  // S3 was paused by operator preference; the gate reopens it only when the
  // operator flips it on explicitly.
  recallGate: false,
  // Warn-only keeps the pre-existing interceptor behavior as the default;
  // stronger modes are the operator's explicit choice.
  immuneMode: "warn",
  fastbrainJudge: "off",
};

export function getGlobalConfigPath(): string {
  return path.join(os.homedir(), ".pi", "agent", "uma.json");
}

export function loadConfig(cwd: string): { config: PluginConfig; globalFile: string } {
  const globalFile = getGlobalConfigPath();
  let merged: PluginConfig = { ...DEFAULT_CONFIG };

  // 1. Load global config
  if (fs.existsSync(globalFile)) {
    try {
      const data = JSON.parse(fs.readFileSync(globalFile, "utf-8"));
      merged = { ...merged, ...data };
    } catch {
      // ignore invalid json
    }
  }

  // 2. Load project config (.pi/uma.json)
  const projectFile = path.join(cwd, ".pi", "uma.json");
  if (fs.existsSync(projectFile)) {
    try {
      const data = JSON.parse(fs.readFileSync(projectFile, "utf-8"));
      merged = { ...merged, ...data };
    } catch {
      // ignore invalid json
    }
  }

  return { config: merged, globalFile };
}

export function saveConfig(
  patch: Partial<PluginConfig>,
  isGlobal: boolean,
  cwd: string,
  globalFile: string
): void {
  const targetFile = isGlobal ? globalFile : path.join(cwd, ".pi", "uma.json");
  const targetDir = path.dirname(targetFile);

  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
  }

  let existing: Record<string, unknown> = {};
  if (fs.existsSync(targetFile)) {
    try {
      existing = JSON.parse(fs.readFileSync(targetFile, "utf-8"));
    } catch {
      existing = {};
    }
  }

  const updated = { ...existing, ...patch };
  fs.writeFileSync(targetFile, JSON.stringify(updated, null, 2), "utf-8");
}
