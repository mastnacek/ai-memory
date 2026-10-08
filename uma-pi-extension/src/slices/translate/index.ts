/**
 * Display translation for the proposal modal.
 *
 * TRANSLATION IS DISPLAY-ONLY: the returned text must never reach the
 * proposal object — the reviewer sees Czech, UMA stores the original
 * English body untouched. That separation is the whole point of this
 * slice; folding the translated body back into the proposal would
 * silently change what gets saved (the modal round-trip lesson).
 */
import type { Api, Model } from "@earendil-works/pi-ai";
import { completeSimple } from "@earendil-works/pi-ai/compat";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";

import { buildTranslateMessages } from "./prompt.js";
export { buildTranslateMessages };

export interface DisplayTranslationResult {
  /** Translated markdown; undefined when translation failed or was skipped. */
  text?: string;
  error?: string;
}

/**
 * Translates a fact body for display. Uses the session's current model —
 * no extra configuration, and the auth comes from pi's own model registry.
 * Degrades to `{}` (no text) on any failure: the modal then simply shows
 * the original, which is always acceptable because translation is a
 * display convenience, never a data transformation.
 */
export async function translateForDisplay(
  ctx: ExtensionContext,
  body: string,
): Promise<DisplayTranslationResult> {
  try {
    const model = ctx.model as Model<Api> | undefined;
    if (!model) return { error: "no active model" };
    const auth = await ctx.modelRegistry.getApiKeyAndHeaders(model);
    if (!auth.ok) return { error: auth.error };
    const reply = await completeSimple(model, buildTranslateMessages(body) as never, {
      apiKey: auth.apiKey,
      headers: auth.headers,
    });
    const text = reply.content
      ?.filter((c): c is { type: "text"; text: string } => c.type === "text")
      .map((c) => c.text)
      .join("")
      .trim();
    if (!text) return { error: "empty translation" };
    return { text };
  } catch (error) {
    return { error: error instanceof Error ? error.message : String(error) };
  }
}

/**
 * Modal-side controller over the display translation lifecycle.
 *
 * Owns the cache and the state machine so the modal keeps only wiring:
 * `start()` fires the translation once (fire-and-forget), `onChange` is the
 * modal's re-render trigger, and `bodyFor()` is the only place the display
 * selection happens — the original body and the translated cache never
 * merge, which is the slice invariant.
 */
export class DisplayTranslation {
  private text?: string;
  private state: "idle" | "pending" | "done" | "failed" = "idle";

  constructor(
    private readonly ctx: ExtensionContext,
    private readonly onChange: () => void,
  ) {}

  /** Fires the translation once; later calls are no-ops. */
  start(body: string): void {
    if (this.state !== "idle") return;
    this.state = "pending";
    void translateForDisplay(this.ctx, body).then((result) => {
      if (result.text) {
        this.text = result.text;
        this.state = "done";
      } else {
        this.state = "failed";
      }
      this.onChange();
    });
  }

  get pending(): boolean {
    return this.state === "pending";
  }

  get failed(): boolean {
    return this.state === "failed";
  }

  /** The body to display: the cache when shown and ready, else the original. */
  bodyFor(original: string, showTranslation: boolean): string {
    return showTranslation && this.state === "done" && this.text !== undefined
      ? this.text
      : original;
  }
}
