/**
 * Display translation for the proposal modal.
 *
 * TRANSLATION IS DISPLAY-ONLY: the returned text must never reach the
 * proposal object — the reviewer sees Czech, UMA stores the original
 * English body untouched. That separation is the whole point of this
 * slice; folding the translated body back into the proposal would
 * silently change what gets saved (the modal round-trip lesson).
 */
import type { Api, AssistantMessage, Model } from "@earendil-works/pi-ai";
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

    // completeSimple reports model errors as a MESSAGE (stopReason "error"),
    // not a thrown exception — extract that path explicitly.
    const extract = (reply: AssistantMessage): DisplayTranslationResult => {
      if (reply.stopReason === "error") {
        return { error: reply.errorMessage ?? "model error" };
      }
      const text = reply.content
        ?.filter((c): c is { type: "text"; text: string } => c.type === "text")
        .map((c) => c.text)
        .join("")
        .trim();
      if (!text) return { error: "empty translation" };
      return { text };
    };

    const opts = {
      apiKey: auth.apiKey,
      headers: auth.headers,
      env: auth.env,
      signal: ctx.signal,
    } as never;

    // Prefer the registry's own completion (it resolves provider routing and
    // auth the way pi does internally), then fall back to the compat shim.
    const registry = ctx.modelRegistry as
      | {
          completeSimple?: (
            m: Model<Api>,
            context: never,
            options?: never,
          ) => Promise<AssistantMessage>;
          complete?: (
            m: Model<Api>,
            context: never,
            options?: never,
          ) => Promise<AssistantMessage>;
        }
      | undefined;
    const context = buildTranslateMessages(body) as never;
    if (registry && typeof registry.completeSimple === "function") {
      return extract(await registry.completeSimple(model, context, opts));
    }
    if (registry && typeof registry.complete === "function") {
      return extract(await registry.complete(model, context, opts));
    }
    return extract(await completeSimple(model, context, opts));
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
 * merge, which is the slice invariant. The failure reason is kept visible:
 * "unavailable" without a why is undebuggable.
 */
export class DisplayTranslation {
  private text?: string;
  private state: "idle" | "pending" | "done" | "failed" = "idle";
  private lastError?: string;

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
        this.lastError = result.error;
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

  get error(): string | undefined {
    return this.lastError;
  }

  /** The body to display: the cache when shown and ready, else the original. */
  bodyFor(original: string, showTranslation: boolean): string {
    return showTranslation && this.state === "done" && this.text !== undefined
      ? this.text
      : original;
  }
}
