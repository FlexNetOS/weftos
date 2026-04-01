/**
 * WeftOS Theme type definitions.
 * Matches the JSON theme schema from theming-system.md v0.2.0.
 */

export type ThemeMode = "dark" | "light";

export interface ThemeColors {
  background: string;
  foreground: string;
  surface: string;
  surfaceAlt?: string;
  border: string;
  borderStrong?: string;

  primary: string;
  primaryForeground?: string;
  secondary: string;
  secondaryForeground?: string;
  accent: string;
  accentForeground?: string;

  muted: string;
  mutedForeground: string;

  success: string;
  successForeground?: string;
  warning: string;
  warningForeground?: string;
  error: string;
  errorForeground?: string;
  info: string;
  infoForeground?: string;

  ring: string;
  selection?: string;

  semantic?: Record<string, string>;
}

export interface ThemeTypography {
  fontFamily: {
    sans: string;
    mono: string;
    heading?: string;
  };
  fontSize: Record<string, string>;
  fontWeight?: Record<string, number>;
  lineHeight?: Record<string, number>;
  letterSpacing?: Record<string, string>;
}

export interface ThemeSpacing {
  unit: number;
  scale: number[];
}

export interface ThemeBorders {
  radius: Record<string, string>;
  width: Record<string, string>;
}

export interface ThemeEffects {
  shadow?: Record<string, string>;
  glow?: Record<string, string>;
  blur?: Record<string, string>;
  opacity?: Record<string, number>;
}

export interface ThemeAnimation {
  duration: Record<string, string>;
  easing: Record<string, string>;
}

export interface AnsiPalette {
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  brightBlack: string;
  brightRed: string;
  brightGreen: string;
  brightYellow: string;
  brightBlue: string;
  brightMagenta: string;
  brightCyan: string;
  brightWhite: string;
}

export interface ConsolePromptToken {
  color: string;
  bold?: boolean;
  italic?: boolean;
  prefix?: string;
  suffix?: string;
  text?: string;
}

export interface ConsolePrompt {
  format: string;
  tokens: Record<string, ConsolePromptToken>;
}

export interface ConsolePanel {
  border: "single" | "double" | "rounded" | "heavy" | "none";
  borderColor: string;
  titleColor: string;
  titleBold: boolean;
  padding: number;
}

export interface ConsoleTable {
  borderStyle: "single" | "double" | "rounded" | "heavy" | "none" | "ascii";
  headerColor: string;
  headerBold: boolean;
  rowAlternateBackground: boolean;
  alternateColor: string;
}

export interface ThemeConsole {
  ansi: AnsiPalette;
  background: string;
  foreground: string;
  cursor: string;
  cursorAccent: string;
  selectionBackground: string;
  prompt?: ConsolePrompt;
  syntaxHighlighting?: Record<string, string>;
  panel?: ConsolePanel;
  table?: ConsoleTable;
}

export interface ThemeHud {
  foreground: string;
  background: string;
  warningForeground: string;
  errorForeground: string;
  mutedForeground: string;
  headerSeparator: string;
  footerSeparator: string;
  selectionIndicator: string;
  progressFilled: string;
  progressEmpty: string;
  progressBrackets: [string, string];
}

export interface ThemeVoice {
  pace: "slow" | "normal" | "fast";
  pitch: "low" | "medium" | "high";
  emphasisStyle: "stress" | "volume" | "pitch";
  pauseAfterHeading: string;
  pauseAfterParagraph: string;
  errorTone: string;
  successTone: string;
  codeReadStyle: "spell" | "summarize" | "skip";
}

export interface WeftOSTheme {
  name: string;
  version: string;
  displayName?: string;
  description?: string;
  author?: string;
  license?: string;
  mode: ThemeMode;

  colors: ThemeColors;
  typography: ThemeTypography;
  spacing?: ThemeSpacing;
  borders?: ThemeBorders;
  effects?: ThemeEffects;
  animation?: ThemeAnimation;
  console?: ThemeConsole;
  hud?: ThemeHud;
  voice?: ThemeVoice;
}
