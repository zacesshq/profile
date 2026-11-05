// WCAG 2.2 relative luminance and contrast ratio, the same formula zprf check uses, so the
// extension never draws a pair the profile's own floors would refuse

export function parseHex(hex: string): [number, number, number] | null {
  const m = /^#([0-9a-fA-F]{6})$/.exec(hex);
  if (!m || m[1] === undefined) return null;
  const n = parseInt(m[1], 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

const toLinear = (v: number): number => {
  const s = v / 255;
  // 0.04045 is the corrected threshold WCAG 2.2 uses
  return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
};

export function luminance(hex: string): number {
  const rgb = parseHex(hex);
  if (!rgb) throw new Error(`not a #rrggbb colour: ${hex}`);
  const [r, g, b] = rgb.map(toLinear) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function contrast(a: string, b: string): number {
  const la = luminance(a);
  const lb = luminance(b);
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

/** Dark when white has more contrast with the colour than black does, as the spec defines it. */
export const isDark = (hex: string): boolean => contrast(hex, "#ffffff") > contrast(hex, "#000000");

/** Rounded down, so a printed ratio never overstates the real one. */
export const shownRatio = (r: number): string => `${(Math.floor(r * 100) / 100).toFixed(2)}:1`;
