/**
 * Shared client for display-only translation — the seam that lets
 * consumers (proposal modal, /uma fact views) use the translate slice
 * without cross-slice imports. Memory, CLI output and stored data are
 * NEVER touched by anything here: translation is presentation only.
 */
export {
  DisplayTranslation,
  translateOutputForDisplay,
} from "../slices/translate/index.js";