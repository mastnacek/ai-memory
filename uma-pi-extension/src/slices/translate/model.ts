/**
 * Translate-model resolution shared with the operator's
 * pi-prompt-translate-czk plugin (option A): the plugin's global config
 * file is the single knob. Pure + node:fs only — value imports from
 * pi-ai stay in index.ts so node --test can load this module.
 */
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import type { Api, Model } from "@earendil-works/pi-ai";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";

/** Same shape the translate plugin uses. */
export type TranslateModelSetting = "current" | "default" | `${string}/${string}`;

const PLUGIN_CONFIG_FILE = "pi-prompt-translate.json";
const PI_SETTINGS_FILE = "settings.json";

/** Reads the translate plugin's global config `translateModel`, if any. */
export function readPluginTranslateModel(
  agentDir: string,
): TranslateModelSetting | undefined {
  try {
    const file = join(agentDir, PLUGIN_CONFIG_FILE);
    if (!existsSync(file)) return undefined;
    const value = (JSON.parse(readFileSync(file, "utf-8")) as {
      translateModel?: unknown;
    }).translateModel;
    if (typeof value !== "string" || value.trim().length === 0) return undefined;
    return value as TranslateModelSetting;
  } catch {
    return undefined;
  }
}

/** Pure parse of a TranslateModelSetting. */
export function parseTranslateModelSetting(setting: TranslateModelSetting):
  | { kind: "current" }
  | { kind: "default" }
  | { kind: "explicit"; provider: string; id: string } {
  if (setting === "current") return { kind: "current" };
  if (setting === "default") return { kind: "default" };
  const slash = setting.indexOf("/");
  if (slash <= 0 || slash === setting.length - 1) return { kind: "current" };
  return {
    kind: "explicit",
    provider: setting.slice(0, slash),
    id: setting.slice(slash + 1),
  };
}

/** Reads pi's defaultProvider/defaultModel from settings.json. */
export function readDefaultModel(
  agentDir: string,
): { provider: string; model: string } | undefined {
  try {
    const file = join(agentDir, PI_SETTINGS_FILE);
    if (!existsSync(file)) return undefined;
    const value = JSON.parse(readFileSync(file, "utf-8")) as {
      defaultProvider?: string;
      defaultModel?: string;
    };
    if (!value.defaultProvider || !value.defaultModel) return undefined;
    return { provider: value.defaultProvider, model: value.defaultModel };
  } catch {
    return undefined;
  }
}

/**
 * Resolves the translate model with plugin-config precedence: the
 * translate plugin's configured model first (one knob, the cheap flash
 * model the operator already maintains), then pi's default model, then
 * the session's current model. Every failure degrades to ctx.model —
 * translation must keep working, only the bill changes.
 */
export async function resolveTranslateModel(
  ctx: ExtensionContext,
): Promise<Model<Api>> {
  const fallback = ctx.model as Model<Api> | undefined;
  const setting = readPluginTranslateModel(
    (ctx as { agentDir?: string }).agentDir ?? "",
  );
  if (setting) {
    const parsed = parseTranslateModelSetting(setting);
    if (parsed.kind === "explicit") {
      const found = ctx.modelRegistry.find(parsed.provider, parsed.id);
      if (found) return found;
    } else if (parsed.kind === "default") {
      const def = readDefaultModel(
        (ctx as { agentDir?: string }).agentDir ?? "",
      );
      if (def) {
        const found = ctx.modelRegistry.find(def.provider, def.model);
        if (found) return found;
      }
    }
  }
  if (fallback) return fallback;
  throw new Error("no active model");
}