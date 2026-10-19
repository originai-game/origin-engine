// The economy's formulas and big numbers in words of the locale.
//
// Prices, rates, milestones and stardust are pure functions of the counts
// and the tuning, so the systems, the HUD, the save's merge and the
// playtests agree on every number.
//
// Big numbers come from the string tables (og-i18n): `num.step` is how
// many digits a unit spans (3 in English: K, M, B…; 4 in Chinese: 万, 亿,
// 万亿…), `num.<k>` the k-th unit, `num.short` how mantissa and unit join
// and `num.sci` the scientific form past the last unit. Below the first
// unit's tenfold a number is whole and grouped by `{n, number}`.

import { og } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import type { Game, Tuning } from "./components.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;

/** Generator i's rate and base price from the tuning. */
export function base(t: TuneV, i: number): { rate: number; cost: number } {
  switch (i) {
    case 0:
      return { rate: t.emberRate, cost: t.emberCost };
    case 1:
      return { rate: t.stormRate, cost: t.stormCost };
    case 2:
      return { rate: t.crystalRate, cost: t.crystalCost };
    default:
      return { rate: t.starRate, cost: t.starCost };
  }
}

export function owned(g: GameV, i: number): number {
  return i === 0 ? g.owned0 : i === 1 ? g.owned1 : i === 2 ? g.owned2 : g.owned3;
}

export function setOwned(g: GameV, i: number, n: number): void {
  if (i === 0) g.owned0 = n;
  else if (i === 1) g.owned1 = n;
  else if (i === 2) g.owned2 = n;
  else g.owned3 = n;
}

/** The next one's price: the base times the growth to the count owned, rounded up. */
export function price(t: TuneV, i: number, n: number): number {
  return Math.ceil(base(t, i).cost * Math.pow(t.costGrowth, n));
}

/** Everything's multiplier from stardust. */
export function dustMultiplier(t: TuneV, dust: number): number {
  return 1 + t.dustBonus * dust;
}

/** What generator i makes a second with n owned: doubled at every milestone. */
export function generatorRate(t: TuneV, i: number, n: number, dust: number): number {
  return base(t, i).rate * n * Math.pow(2, Math.floor(n / t.milestone)) * dustMultiplier(t, dust);
}

export function totalRate(t: TuneV, counts: readonly number[], dust: number): number {
  let r = 0;
  counts.forEach((n, i) => (r += generatorRate(t, i, n, dust)));
  return r;
}

/** A tap: the base, plus a share of the rate, times the stardust. */
export function tapValue(t: TuneV, rate: number, dust: number): number {
  return (t.tapBase * dustMultiplier(t, dust)) + t.tapShare * rate;
}

/** Stardust a supernova would give now. */
export function novaGain(t: TuneV, runEarned: number): number {
  return Math.floor(Math.sqrt(runEarned / t.novaScale));
}

/** Run sparks at which the supernova gives its first stardust. */
export function novaAt(t: TuneV): number {
  return t.novaScale;
}

/** Sparks made while the game was closed for `away` seconds. */
export function offlineGain(t: TuneV, rate: number, away: number): number {
  return rate * t.offlineShare * Math.min(Math.max(0, away), t.offlineCap);
}

/** Three significant digits, as text (the mantissa of a big number). */
function mantissa(m: number): string {
  return m >= 100 ? m.toFixed(0) : m >= 10 ? m.toFixed(1) : m.toFixed(2);
}

/** The locale's unit for a big number: its value in units, the unit's text and the decimals to show. */
export function bigParts(world: World, n: number): { value: number; unit: string; decimals: number } {
  const step = Number(og.i18n.t(world, "num.step")) || 3;
  const unitBase = Math.pow(10, step);
  if (!Number.isFinite(n) || n < unitBase * 10) return { value: Math.floor(Math.max(0, n)), unit: "", decimals: 0 };
  let k = Math.floor(Math.log10(n) / step);
  let value = n / Math.pow(unitBase, k);
  // Rounding to three digits can make 1000K: then it is 1.00M.
  if (Number(mantissa(value)) >= unitBase) {
    k += 1;
    value /= unitBase;
  }
  const key = `num.${k}`;
  if (!og.i18n.has(world, key)) return { value, unit: "", decimals: -1 };
  return { value, unit: og.i18n.t(world, key), decimals: value >= 100 ? 0 : value >= 10 ? 1 : 2 };
}

/** A number in the locale's words: `9,999`, `12.3K`, `1.20亿`, `4.56e45`. */
export function big(world: World, n: number): string {
  const p = bigParts(world, n);
  if (p.unit === "" && p.decimals === 0) return og.i18n.t(world, "num.whole", { n: p.value });
  if (p.decimals < 0) {
    const e = Math.floor(Math.log10(n));
    return og.i18n.t(world, "num.sci", { m: mantissa(n / Math.pow(10, e)), e });
  }
  return og.i18n.t(world, "num.short", { m: mantissa(p.value), u: p.unit });
}

/** A duration in the locale's words. */
export function duration(world: World, seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (h > 0) return og.i18n.t(world, "time.hm", { h, m });
  return og.i18n.t(world, "time.ms", { m, s: s % 60 });
}
