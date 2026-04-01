import type { AnsiPalette, WeftOSTheme } from "./types.ts";

/**
 * ANSI 16-color index mapping.
 * Maps theme ANSI palette entries to standard terminal color indices.
 */
export const ANSI_INDEX: Record<keyof AnsiPalette, number> = {
  black: 0,
  red: 1,
  green: 2,
  yellow: 3,
  blue: 4,
  magenta: 5,
  cyan: 6,
  white: 7,
  brightBlack: 8,
  brightRed: 9,
  brightGreen: 10,
  brightYellow: 11,
  brightBlue: 12,
  brightMagenta: 13,
  brightCyan: 14,
  brightWhite: 15,
};

/**
 * Build an xterm.js-compatible ITheme object from a WeftOS theme.
 * Returns undefined if the theme has no console section.
 */
export function toXtermTheme(
  theme: WeftOSTheme,
): Record<string, string> | undefined {
  const con = theme.console;
  if (!con) return undefined;

  return {
    background: con.background,
    foreground: con.foreground,
    cursor: con.cursor,
    cursorAccent: con.cursorAccent,
    selectionBackground: con.selectionBackground,
    black: con.ansi.black,
    red: con.ansi.red,
    green: con.ansi.green,
    yellow: con.ansi.yellow,
    blue: con.ansi.blue,
    magenta: con.ansi.magenta,
    cyan: con.ansi.cyan,
    white: con.ansi.white,
    brightBlack: con.ansi.brightBlack,
    brightRed: con.ansi.brightRed,
    brightGreen: con.ansi.brightGreen,
    brightYellow: con.ansi.brightYellow,
    brightBlue: con.ansi.brightBlue,
    brightMagenta: con.ansi.brightMagenta,
    brightCyan: con.ansi.brightCyan,
    brightWhite: con.ansi.brightWhite,
  };
}

/**
 * Convert an ANSI palette to a 16-element array ordered by index.
 * Useful for terminal emulators that take an array of 16 colors.
 */
export function ansiPaletteToArray(palette: AnsiPalette): string[] {
  const result: string[] = new Array(16);
  for (const [name, index] of Object.entries(ANSI_INDEX)) {
    result[index] = palette[name as keyof AnsiPalette];
  }
  return result;
}

/**
 * Map semantic theme colors to the nearest ANSI color name.
 * Used when rendering themed block output in a 16-color terminal.
 */
export function semanticToAnsi(
  theme: WeftOSTheme,
): Record<string, keyof AnsiPalette> {
  return {
    success: "green",
    warning: "yellow",
    error: "red",
    info: "cyan",
    primary: "blue",
    secondary: "magenta",
    muted: "brightBlack",
    foreground: "white",
    background: "black",
    agentRunning: "green",
    agentIdle: "brightBlack",
    agentStopped: "red",
    governancePermit: "green",
    governanceDeny: "red",
    metricNormal: theme.mode === "dark" ? "white" : "black",
    metricWarn: "yellow",
    metricCrit: "red",
    chainEvent: theme.mode === "dark" ? "brightCyan" : "blue",
  };
}
