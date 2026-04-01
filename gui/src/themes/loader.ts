import type { WeftOSTheme } from "./types.ts";

import oceanDark from "./ocean-dark.json";
import midnight from "./midnight.json";
import paperLight from "./paper-light.json";
import highContrast from "./high-contrast.json";

/**
 * Registry of built-in themes keyed by name.
 */
const BUILTIN_THEMES: Record<string, WeftOSTheme> = {
  "ocean-dark": oceanDark as unknown as WeftOSTheme,
  midnight: midnight as unknown as WeftOSTheme,
  "paper-light": paperLight as unknown as WeftOSTheme,
  "high-contrast": highContrast as unknown as WeftOSTheme,
};

/** All built-in theme names. */
export const BUILTIN_THEME_NAMES = Object.keys(BUILTIN_THEMES);

/**
 * Load a built-in theme by name.
 * Returns undefined if no built-in theme matches.
 */
export function loadBuiltinTheme(name: string): WeftOSTheme | undefined {
  return BUILTIN_THEMES[name];
}

/**
 * Parse a user-provided JSON string into a WeftOSTheme.
 * Performs basic structural validation.
 */
export function parseThemeJSON(json: string): WeftOSTheme {
  const parsed: unknown = JSON.parse(json);
  if (!isValidTheme(parsed)) {
    throw new Error("Invalid theme JSON: missing required fields");
  }
  return parsed;
}

/**
 * Load a user theme from a File object (e.g. from an <input type="file">).
 */
export async function loadThemeFromFile(file: File): Promise<WeftOSTheme> {
  const text = await file.text();
  return parseThemeJSON(text);
}

/**
 * Resolve a theme by name using the discovery chain:
 * workspace > user > built-in.
 *
 * Currently only built-in themes are supported. User and workspace
 * theme discovery will be added when the filesystem API is available
 * (e.g. via Tauri fs plugin).
 */
export function resolveTheme(name: string): WeftOSTheme | undefined {
  // Future: check workspace .weftos/themes/ first
  // Future: check user ~/.weftos/themes/ second
  return loadBuiltinTheme(name);
}

function isValidTheme(value: unknown): value is WeftOSTheme {
  if (typeof value !== "object" || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.name === "string" &&
    typeof obj.mode === "string" &&
    (obj.mode === "dark" || obj.mode === "light") &&
    typeof obj.colors === "object" &&
    obj.colors !== null &&
    typeof obj.typography === "object" &&
    obj.typography !== null
  );
}
