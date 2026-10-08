/**
 * Pure translate prompt — import-free so `node --test` (strip-types) can
 * exercise the contract directly (same pattern as fastbrain/policy.ts).
 */

/**
 * Pure prompt builder (unit-tested): the model gets the fact's markdown and
 * must return ONLY the Czech markdown — same structure, headings, lists,
 * tags, ULIDs, code spans and file paths preserved verbatim, prose
 * translated. No preamble, no commentary, no code fences around the output.
 */
export function buildTranslateMessages(body: string): Array<{
  role: "system" | "user";
  content: string;
}> {
  const system = [
    "You are a translation engine for AI-memory fact records.",
    "Translate the user's Markdown text into Czech.",
    "Preserve exactly: Markdown structure (## / ### headings, lists, blank lines),",
    "code spans and code blocks, file paths, identifiers, ULID identifiers,",
    "tag names (#tag), and technical terms that are commonly used in English.",
    "Translate only the prose.",
    "Return ONLY the translated Markdown — no preamble, no commentary, no code fence around the whole output.",
  ].join(" ");
  return [
    { role: "system", content: system },
    { role: "user", content: body },
  ];
}

