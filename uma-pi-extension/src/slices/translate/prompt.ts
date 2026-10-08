/**
 * Pure translate prompt — import-free so `node --test` (strip-types) can
 * exercise the contract directly (same pattern as fastbrain/policy.ts).
 */

/** The pi-ai `Context` shape this builder produces (structural, no import):
 * a `UserMessage` carries a `timestamp`, and a message missing it fails
 * the real `Message` assignment — exactly what the first version got
 * wrong twice (bare array, then timestamp-less messages). */
export interface TranslateContext {
  systemPrompt: string;
  messages: Array<{ role: "user"; content: string; timestamp: number }>;
}

/**
 * Pure prompt builder (unit-tested): the model gets the fact's markdown and
 * must return ONLY the Czech markdown — same structure, headings, lists,
 * tags, ULIDs, code spans and file paths preserved verbatim, prose
 * translated. No preamble, no commentary, no code fence around the output.
 *
 * The result MUST be a `Context` object — `{ systemPrompt, messages }` —
 * not a bare message array: a bare array satisfies the parameter only
 * through a cast, and then `context.messages` is undefined and the request
 * ships with no messages at all (the bug that made translation silently
 * "unavailable").
 */
export function buildTranslateContext(body: string): TranslateContext {
  const systemPrompt = [
    "You are a translation engine for AI-memory fact records.",
    "Translate the user's Markdown text into Czech.",
    "Preserve exactly: Markdown structure (## / ### headings, lists, blank lines),",
    "code spans and code blocks, file paths, identifiers, ULID identifiers,",
    "tag names (#tag), and technical terms that are commonly used in English.",
    "Translate only the prose.",
    "Return ONLY the translated Markdown — no preamble, no commentary, no code fence around the whole output.",
  ].join(" ");
  return {
    systemPrompt,
    messages: [{ role: "user", content: body, timestamp: Date.now() }],
  };
}
