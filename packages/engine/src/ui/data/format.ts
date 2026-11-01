// Numbers as data views show them: grouped digits, percentages, compact
// units (12.4K, 3.40M), with a true minus sign so columns of signed
// values line up in the mono face.

/** How a number reads: `grouped` 12,840 (the default), `percent` (0.42 is 42%), `compact` 12.4K, `plain`. */
export type NumberFormat = "grouped" | "percent" | "compact" | "plain";

export function groupNumber(v: number, digits = 0): string {
  if (!Number.isFinite(v)) return "—";
  const s = Math.abs(v).toFixed(digits);
  const [w, f] = s.split(".");
  const g = w!.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  const neg = v < 0 && Number(s) !== 0;
  return (neg ? "−" : "") + g + (f ? `.${f}` : "");
}

export function compactNumber(v: number): string {
  const a = Math.abs(v);
  const u: Array<[number, string]> = [[1e12, "T"], [1e9, "B"], [1e6, "M"], [1e3, "K"]];
  for (const [k, s] of u) {
    if (a >= k) {
      const x = a / k;
      return `${v < 0 ? "−" : ""}${x.toFixed(x < 10 ? 2 : x < 100 ? 1 : 0)}${s}`;
    }
  }
  return groupNumber(v, a < 10 && a !== Math.round(a) ? 1 : 0);
}

/** `v` in `format` with `digits` decimals (grouped, percent) and a unit after it. */
export function formatNumber(v: number, format: string | undefined, digits = 0, unit = ""): string {
  let s: string;
  if (format === "percent") s = `${groupNumber(v * 100, digits)}%`;
  else if (format === "compact") s = compactNumber(v);
  else if (format === "plain") s = Number.isFinite(v) ? String(v) : "—";
  else s = groupNumber(v, digits);
  return unit ? `${s}${unit.startsWith("%") ? "" : " "}${unit}` : s;
}

/** A signed delta: +12, −3.5, ±0. */
export function formatDelta(v: number, format: string | undefined, digits = 0, unit = ""): string {
  if (Math.abs(v) < 0.5 * 10 ** -digits * (format === "percent" ? 0.01 : 1)) return `±${formatNumber(0, format, digits, unit)}`;
  return `${v > 0 ? "+" : ""}${formatNumber(v, format, digits, unit)}`;
}

/**
 * Axis ticks over [lo, hi]: about `count` round steps (1, 2 or 5
 * times a power of ten), the range widened to whole steps.
 */
export function niceTicks(lo: number, hi: number, count: number): { lo: number; hi: number; step: number; ticks: number[] } {
  if (!Number.isFinite(lo) || !Number.isFinite(hi)) return { lo: 0, hi: 1, step: 1, ticks: [0, 1] };
  if (hi - lo < 1e-9) {
    const pad = Math.abs(hi) > 0 ? Math.abs(hi) * 0.5 : 1;
    if (lo >= 0 && lo - pad < 0) lo = 0;
    else lo -= pad;
    hi += pad;
  }
  const raw = (hi - lo) / Math.max(1, count);
  const mag = 10 ** Math.floor(Math.log10(raw));
  const f = raw / mag;
  // d3's thresholds: the step nearest the raw one on a log scale.
  const step = (f >= Math.sqrt(50) ? 10 : f >= Math.sqrt(10) ? 5 : f >= Math.SQRT2 ? 2 : 1) * mag;
  const a = Math.floor(lo / step + 1e-9) * step;
  const b = Math.ceil(hi / step - 1e-9) * step;
  const ticks: number[] = [];
  for (let v = a; v <= b + step * 1e-6; v += step) ticks.push(Math.abs(v) < step * 1e-9 ? 0 : v);
  return { lo: a, hi: b, step, ticks };
}
