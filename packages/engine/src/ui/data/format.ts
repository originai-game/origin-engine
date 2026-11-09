// Numbers as data views show them: grouped digits, percentages, compact
// units (12.4K, 3.40M), with a true minus sign so columns of signed
// values line up in the mono face.

import type * as Api from "@origingame/engine";
import type { DateStyle } from "@origingame/engine";
import { i18n } from "../../e6.ts";
import { currentCall } from "../../runtime.ts";

const world = () => ({ id: currentCall()?.engine ?? 0 });

export function decimalNumber(v: number, decimals?: number, grouping = true): string {
  return Number.isFinite(v) ? i18n.formatNumber(world(), v, { decimals, grouping }) : "—";
}

export const localizedDuration = (seconds: number, short = false): string => i18n.formatDuration(world(), seconds, short ? "short" : "clock");

/** How a number reads: `grouped` 12,840 (the default), `percent` (0.42 is 42%), `compact` 12.4K, `plain`. */
export type NumberFormat = "grouped" | "percent" | "compact" | "plain";

export function groupNumber(v: number, digits = 0): string {
  return decimalNumber(v, digits).replace(/^-/, "−");
}

export function compactNumber(v: number, decimals?: number): string {
  return Number.isFinite(v) ? i18n.formatCompact(world(), v, decimals) : "—";
}

/** `v` in `format` with `digits` decimals (grouped, percent) and a unit after it. */
export function formatNumber(v: number, format: string | undefined, digits = 0, unit = ""): string {
  let s: string;
  if (format === "percent") s = `${groupNumber(v * 100, digits)}%`;
  else if (format === "compact") s = compactNumber(v);
  else if (format === "plain") s = decimalNumber(v, undefined, false);
  else s = groupNumber(v, digits);
  return unit ? `${s}${unit.startsWith("%") ? "" : " "}${unit}` : s;
}

// ── numbers ───────────────────────────────────────────────────────────────

const SHORT = ["", "K", "M", "B", "T"];

/** Idle letters after T: aa, ab, … az, ba, … zz, aaa … */
function letters(n: number): string {
  let k = n;
  let s = "";
  let len = 2;
  let span = 26 ** len;
  while (k >= span) {
    k -= span;
    len += 1;
    span = 26 ** len;
  }
  for (let i = 0; i < len; i++) {
    s = String.fromCharCode(97 + (k % 26)) + s;
    k = Math.floor(k / 26);
  }
  return s;
}

function group(intText: string, sep: string): string {
  return sep ? intText.replace(/\B(?=(\d{3})+(?!\d))/g, sep) : intText;
}

function fixed(v: number, decimals: number, point: string): string {
  const s = v.toFixed(decimals);
  return point === "." ? s : s.replace(".", point);
}

/** Three significant digits: 1.23, 12.3, 123. */
function sig3(m: number, decimals: number | undefined): number {
  if (decimals !== undefined) return decimals;
  return m < 10 ? 2 : m < 100 ? 1 : 0;
}

/**
 * og.ui.formatNumber: a number in a notation. Exact for the integers a
 * double holds; past 1e308 through `exponent`. Ordinary formats use
 * og.i18n's shared CLDR data; explicit custom notation stays deterministic.
 */
export function formatNotation(value: number, format: Api.NumberFormatDesc = {}): string {
  const notation = format.notation ?? "grouped";
  const sep = format.group ?? ",";
  const point = format.point ?? ".";
  const pre = format.prefix ?? "";
  const post = format.suffix ?? "";
  if (!Number.isFinite(value)) return `${pre}—${post}`;
  const dec = format.decimals ?? undefined;
  const shift = format.exponent ?? 0;
  const custom = format.group != null || format.point != null;
  const scaled = value * 10 ** shift;
  if (!custom && Number.isFinite(scaled) && (notation === "grouped" || notation === "compact" && Math.abs(scaled) < 1e15)) {
    const body = notation === "compact" ? compactNumber(scaled, dec) : decimalNumber(scaled, dec ?? 0);
    const positive = format.sign && scaled > 0 && body !== (notation === "compact" ? compactNumber(0, dec) : decimalNumber(0, dec ?? 0)) ? "+" : "";
    return `${pre}${positive}${body}${post}`;
  }
  // The value as a mantissa in [1, 10) and a base-10 exponent.
  let mant = Math.abs(value);
  let exp10 = 0;
  if (mant > 0) {
    const k = Math.floor(Math.log10(mant));
    mant /= 10 ** k;
    exp10 = k + shift;
    if (mant >= 10) {
      mant /= 10;
      exp10 += 1;
    } else if (mant < 1) {
      mant *= 10;
      exp10 -= 1;
    }
  }
  const numeral = (n: number, d: number, grouping = false) => decimalNumber(value < 0 ? -n : n, d, grouping);
  const sci = (m: number, e: number, d: number) => custom
    ? `${fixed(m, d, point)}e${e}`
    : `${numeral(m, d)}e${decimalNumber(e, 0, false)}`;
  let body: string;
  if (notation === "grouped" || exp10 < 4) {
    if (exp10 >= 21) body = sci(mant, exp10, dec ?? 2);
    else {
      const plain = shift === 0 ? Math.abs(value) : mant * 10 ** exp10;
      const [i, f] = fixed(plain, dec ?? 0, ".").split(".");
      body = custom ? `${group(i ?? "0", sep)}${f !== undefined ? point + f : ""}` : numeral(plain, dec ?? 0, true);
    }
  } else if (notation === "scientific") body = sci(mant, exp10, dec ?? 2);
  else if (notation === "engineering") {
    const k = Math.floor(exp10 / 3) * 3;
    const em = mant * 10 ** (exp10 - k);
    body = sci(em, k, sig3(em, dec));
  } else {
    // compact and letters: groups of thousands.
    let tier = Math.floor(exp10 / 3);
    let tm = mant * 10 ** (exp10 - tier * 3);
    let d = sig3(tm, dec);
    // 999.95 rounds up into the next group.
    if (Number(tm.toFixed(d)) >= 1000) {
      tier += 1;
      tm /= 1000;
      d = sig3(tm, dec);
    }
    if (tier < SHORT.length) body = (custom ? fixed(tm, d, point) : numeral(tm, d)) + SHORT[tier]!;
    else if (notation === "letters") body = (custom ? fixed(tm, d, point) : numeral(tm, d)) + letters(tier - SHORT.length);
    else body = sci(mant, exp10, dec ?? 2);
  }
  const zero = value === 0 || (exp10 < 0 && Number(Math.abs(scaled).toFixed(dec ?? 0)) === 0);
  const sign = custom && value < 0 && !zero ? "-" : format.sign && value > 0 && !zero ? "+" : "";
  return `${pre}${sign}${body}${post}`;
}
