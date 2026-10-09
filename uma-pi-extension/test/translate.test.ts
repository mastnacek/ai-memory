import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  parseTranslateModelSetting,
  readDefaultModel,
  readPluginTranslateModel,
} from "../src/slices/translate/model.ts";
import { buildTranslateContext } from "../src/slices/translate/prompt.ts";

test("plugin config is the shared translate-model knob", () => {
  const dir = mkdtempSync(join(tmpdir(), "uma-tr-"));
  try {
    assert.equal(readPluginTranslateModel(dir), undefined);
    writeFileSync(
      join(dir, "pi-prompt-translate.json"),
      JSON.stringify({ translateModel: "openrouter/google/gemini-3.5-flash-lite" }),
    );
    assert.equal(readPluginTranslateModel(dir), "openrouter/google/gemini-3.5-flash-lite");
    writeFileSync(join(dir, "pi-prompt-translate.json"), "{ broken");
    assert.equal(readPluginTranslateModel(dir), undefined);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("setting parsing: current, default, explicit, malformed", () => {
  assert.deepEqual(parseTranslateModelSetting("current"), { kind: "current" });
  assert.deepEqual(parseTranslateModelSetting("default"), { kind: "default" });
  assert.deepEqual(parseTranslateModelSetting("openrouter/google/gemini-3.5-flash-lite"), {
    kind: "explicit",
    provider: "openrouter",
    id: "google/gemini-3.5-flash-lite",
  });
  assert.deepEqual(parseTranslateModelSetting("nobase/"), { kind: "current" });
});

test("pi default model is read from settings.json", () => {
  const dir = mkdtempSync(join(tmpdir(), "uma-tr-"));
  try {
    assert.equal(readDefaultModel(dir), undefined);
    writeFileSync(
      join(dir, "settings.json"),
      JSON.stringify({ defaultProvider: "openrouter", defaultModel: "z-ai/glm-5.3-flash" }),
    );
    assert.deepEqual(readDefaultModel(dir), { provider: "openrouter", model: "z-ai/glm-5.3-flash" });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("translate context keeps the real Context shape", () => {
  const ctx = buildTranslateContext("Body text longer than a tweet.");
  assert.ok(ctx.systemPrompt.length > 50);
  assert.equal(ctx.messages.length, 1);
  assert.equal(ctx.messages[0].role, "user");
  assert.ok(typeof ctx.messages[0].timestamp === "number");
});
