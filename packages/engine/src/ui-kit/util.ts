// Small helpers the screen templates share: sizes, insets, the frame's
// tokens, number and time formats, and HUD capsules. Nothing here keeps
// state: what must persist lives in the UI's memory (og.ui.memory).

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";

export const px = (n: number): Api.Size => ({ kind: "fixed", px: n });
export const grow = (min?: number, max?: number): Api.Size => ({ kind: "grow", ...(min === undefined ? {} : { min }), ...(max === undefined ? {} : { max }) });
export const fit = (min?: number, max?: number): Api.Size => ({ kind: "fit", ...(min === undefined ? {} : { min }), ...(max === undefined ? {} : { max }) });
export const pct = (fraction: number): Api.Size => ({ kind: "percent", fraction });
export const all = (n: number): Api.Insets => ({ left: n, top: n, right: n, bottom: n });
export const xy = (x: number, y: number): Api.Insets => ({ left: x, top: y, right: x, bottom: y });
export const token = (name: Api.ThemeColor, alpha?: number): Api.Paint => (alpha === undefined ? { token: name } : { token: name, alpha });

/** The tokens of the frame being built, read once per UI tick. */
let cached: { tick: number; theme: Api.UiTheme; viewport: Api.Viewport } | null = null;
export function frame(): { theme: Api.UiTheme; viewport: Api.Viewport; t: Tokens; phone: boolean; touch: boolean } {
  const tick = kit.tick();
  if (!cached || cached.tick !== tick) cached = { tick, theme: kit.frameTheme(), viewport: kit.frameViewport() };
  const { theme, viewport } = cached;
  const phone = viewport.breakpoint === "phonePortrait" || viewport.breakpoint === "phoneLandscape";
  return { theme, viewport, t: tokens(theme), phone, touch: viewport.coarsePointer || phone || viewport.breakpoint === "tablet" };
}

export interface Tokens {
  xxs: number; xs: number; sm: number; md: number; lg: number; xl: number; xxl: number; xl3: number;
  small: number; control: number; bar: number; panel: number;
  controlSm: number; controlMd: number; controlLg: number; gutter: number; row: number;
}

function tokens(t: Api.UiTheme): Tokens {
  const s = (k: string): number => t.space[k] ?? 0;
  const r = (k: string): number => t.radius[k] ?? 0;
  return {
    xxs: s("xxs"), xs: s("xs"), sm: s("sm"), md: s("md"), lg: s("lg"), xl: s("xl"), xxl: s("xxl"), xl3: s("xl3"),
    small: r("small"), control: r("control"), bar: r("bar"), panel: r("panel"),
    controlSm: t.sizes.controlSm, controlMd: t.sizes.controlMd, controlLg: t.sizes.controlLg, gutter: t.sizes.gutter, row: t.sizes.row,
  };
}

/** A count for a HUD: 12,840 below a million, then 3.40M, 1.20B, 7.00T, then 1.2e15. */
export function fmt(n: number): string {
  if (!Number.isFinite(n)) return "—";
  const a = Math.abs(n);
  const sign = n < 0 ? "-" : "";
  if (a < 1_000_000) return sign + Math.floor(a).toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  const units: Array<[number, string]> = [[1e12, "T"], [1e9, "B"], [1e6, "M"]];
  for (const [v, u] of units) {
    if (a >= v && a < v * 1000) {
      const x = a / v;
      return sign + (x < 100 ? x.toFixed(x < 10 ? 2 : 1) : Math.floor(x).toString()) + u;
    }
  }
  return sign + a.toExponential(1).replace("+", "");
}

/** Seconds as m:ss (or m:ss.mmm with `millis`); negative is 0:00. */
export function clock(seconds: number, millis = false): string {
  const s = Math.max(0, seconds);
  const m = Math.floor(s / 60);
  const rest = s - m * 60;
  const whole = Math.floor(rest);
  const head = `${m}:${whole.toString().padStart(2, "0")}`;
  return millis ? `${head}.${Math.floor((rest - whole) * 1000).toString().padStart(3, "0")}` : head;
}

/** A finished time as m:ss.mmm, cut to the millisecond as race timing is, but never a millisecond short by a float's last bit (96.233 s is 1:36.233, where 96.233 * 1000 is 96232.99999999999); negative is 0:00.000. */
export function lapTime(seconds: number): string {
  const ms = Math.floor(Math.max(0, seconds) * 1000 + 1e-6);
  const m = Math.floor(ms / 60000);
  const s = Math.floor(ms / 1000) % 60;
  return `${m}:${s.toString().padStart(2, "0")}.${(ms % 1000).toString().padStart(3, "0")}`;
}

/** A HUD capsule: a translucent dark pill (material-thick) that keeps text legible over any art. */
export function capsule(id: string, build: () => void, desc: { gap?: number; padX?: number; height?: number } = {}): Api.Response {
  const { t } = frame();
  return kit.row(id, {
    padding: xy(desc.padX ?? t.md, 0),
    height: px(desc.height ?? t.controlMd),
    gap: desc.gap ?? t.sm,
    cross: "center",
    background: token("hud"),
    radius: t.control,
    frame: "hud",
  }, build) as Api.Response;
}

/** An icon of the atlas, or a small solid mark of the same ink when the atlas lacks it; `label` names it for a screen reader (IconDesc.label: empty is decoration). */
export function iconOr(id: string, name: string, size: number, tint: Api.Paint, label?: string): void {
  if (kit.hasIcon(name)) kit.icon(id, name, label === undefined ? { size, tint } : { size, tint, label });
  else kit.image(id, { tint, width: px(size * 0.6), height: px(size * 0.6), label: label ?? "" });
}

/** Menu items with defaults filled in, the first primary unless one is. */
export function menuItems(items: readonly Api.MenuItem[] | undefined, fallback: ReadonlyArray<[string, string]>): Required<Pick<Api.MenuItem, "id" | "label" | "disabled" | "primary">>[] {
  const list: readonly Api.MenuItem[] = items && items.length > 0 ? items : fallback.map(([id, label]) => ({ id, label }));
  const anyPrimary = list.some((i) => i.primary);
  return list.map((i, k) => ({
    id: i.id ?? `item${k}`,
    label: i.label ?? i.id ?? "",
    disabled: i.disabled ?? false,
    primary: anyPrimary ? (i.primary ?? false) : k === 0,
  }));
}
