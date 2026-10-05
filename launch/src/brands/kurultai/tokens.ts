/**
 * Kurultai design tokens — deep-black brain canvas, white + slight purple.
 * Source of truth: website/src styles (the Brain's three-color rule:
 * deep black, white, slight purple — semantic hues only for status).
 */
export const color = {
  canvas: '#050508',
  surface: '#13101D',
  raised: '#1C1630',
  hairline: '#222230',
  ink: '#F4F0FF',
  ink2: '#8A82A8',
  ink3: '#6B6480',
  accent: '#A855F7',
  accentSoft: '#C084FC',
  accentDeep: '#7C3AED',
  accentTint: '#1C1630',
  surfaceSunk: '#090711',
  onAccent: '#050508',
  passed: '#7FD4A8',
  passedTint: '#10241C',
  failed: '#FF6B8A',
  failedTint: '#2A1418',
  caution: '#FBBF24',
  cautionTint: '#2A2010',
} as const;

export const motion = {
  instant: 100,
  state: 180,
  panel: 260,
  easeOut: [0.2, 0, 0, 1] as const,
  easeInOut: [0.4, 0, 0.2, 1] as const,
} as const;

export const radius = {sm: 6, md: 10, lg: 14} as const;

export const font = {
  grotesk: "'Schibsted Grotesk', sans-serif",
  mono: "'Martian Mono', monospace",
} as const;

export const statusColor = {
  trusted: color.passed,
  quarantine: color.caution,
  pending: color.accentSoft,
  failed: color.failed,
} as const;

export type StatusName = keyof typeof statusColor;
