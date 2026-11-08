// Numbers as data views show them: grouped digits, percentages, compact
// units (12.4K, 3.40M), with a true minus sign so columns of signed
// values line up in the mono face.

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
