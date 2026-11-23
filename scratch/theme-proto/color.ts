// OKLCH <-> sRGB and WCAG contrast, for the theme prototype's colour stage.
export type Lch = { l: number; c: number; h: number; a?: number };

const lin = (u: number) => (u <= 0.04045 ? u / 12.92 : ((u + 0.055) / 1.055) ** 2.4);
const enc = (u: number) => (u <= 0.0031308 ? 12.92 * u : 1.055 * u ** (1 / 2.4) - 0.055);

export function hexToRgb(hex: string): [number, number, number, number] {
  const h = hex.replace("#", "");
  const n = (i: number) => parseInt(h.slice(i, i + 2), 16) / 255;
  return [n(0), n(2), n(4), h.length === 8 ? n(6) : 1];
}

export function rgbToOklch(r: number, g: number, b: number): Lch {
  const [R, G, B] = [lin(r), lin(g), lin(b)];
  const l = Math.cbrt(0.4122214708 * R + 0.5363325363 * G + 0.0514459929 * B);
  const m = Math.cbrt(0.2119034982 * R + 0.6806995451 * G + 0.1073969566 * B);
  const s = Math.cbrt(0.0883024619 * R + 0.2817188376 * G + 0.6299787005 * B);
  const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const Bb = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  return { l: L, c: Math.hypot(A, Bb), h: ((Math.atan2(Bb, A) * 180) / Math.PI + 360) % 360 };
}

export function hexToOklch(hex: string): Lch {
  const [r, g, b, a] = hexToRgb(hex);
  return { ...rgbToOklch(r, g, b), a };
}

function oklchToLinear({ l, c, h }: Lch): [number, number, number] {
  const A = c * Math.cos((h * Math.PI) / 180);
  const B = c * Math.sin((h * Math.PI) / 180);
  const l_ = (l + 0.3963377774 * A + 0.2158037573 * B) ** 3;
  const m_ = (l - 0.1055613458 * A - 0.0638541728 * B) ** 3;
  const s_ = (l - 0.0894841775 * A - 1.291485548 * B) ** 3;
  return [
    4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
    -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
    -0.0041960863 * l_ - 0.7034186147 * m_ + 1.707614701 * s_,
  ];
}

/** Into sRGB: chroma reduced (hue and lightness kept) until the colour fits, as CSS gamut mapping does. */
export function toHex(c: Lch): string {
  let lo = 0, hi = c.c, rgb = oklchToLinear(c);
  const inside = (v: number[]) => v.every((x) => x >= -1e-4 && x <= 1 + 1e-4);
  if (!inside(rgb)) {
    for (let i = 0; i < 24; i++) {
      const mid = (lo + hi) / 2;
      if (inside(oklchToLinear({ ...c, c: mid }))) lo = mid; else hi = mid;
    }
    rgb = oklchToLinear({ ...c, c: lo });
  }
  const b = (x: number) => Math.round(Math.min(1, Math.max(0, enc(Math.min(1, Math.max(0, x))))) * 255).toString(16).padStart(2, "0");
  const a = c.a === undefined || c.a >= 1 ? "" : Math.round(c.a * 255).toString(16).padStart(2, "0");
  return `#${b(rgb[0])}${b(rgb[1])}${b(rgb[2])}${a}`;
}

export function luminance(hex: string): number {
  const [r, g, b] = hexToRgb(hex);
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

export function contrast(a: string, b: string): number {
  const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
}

/**
 * The ink nearest `want` in lightness that reads at `ratio` on every ground, its hue and chroma
 * kept: the contrast guarantee (Material's DynamicColor tone search, in OKLCH lightness).
 */
export function ensure(want: Lch, grounds: string[], ratio: number): Lch {
  const ok = (l: number) => grounds.every((g) => contrast(toHex({ ...want, l }), g) >= ratio);
  if (ok(want.l)) return want;
  let best: number | undefined;
  for (let i = 1; i <= 100; i++) {
    for (const l of [want.l - i / 100, want.l + i / 100]) {
      if (l >= 0 && l <= 1 && ok(l)) { best = l; break; }
    }
    if (best !== undefined) break;
  }
  return { ...want, l: best ?? (luminance(grounds[0]!) > 0.18 ? 0 : 1) };
}
