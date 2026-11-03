// Small helpers the components share: sizes, insets and the frame's
// tokens. Nothing here keeps state: what must persist lives in the UI's
// memory (og.ui.memory).

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";

export const px = (n: number): Api.Size => ({ kind: "fixed", px: n });
export const grow = (min?: number, max?: number): Api.Size => ({ kind: "grow", ...(min === undefined ? {} : { min }), ...(max === undefined ? {} : { max }) });
export const fit = (min?: number, max?: number): Api.Size => ({ kind: "fit", ...(min === undefined ? {} : { min }), ...(max === undefined ? {} : { max }) });
export const pct = (fraction: number): Api.Size => ({ kind: "percent", fraction });
export const all = (n: number): Api.Insets => ({ left: n, top: n, right: n, bottom: n });
export const xy = (x: number, y: number): Api.Insets => ({ left: x, top: y, right: x, bottom: y });
export const token = (name: Api.ThemeColor, alpha?: number): Api.Paint => (alpha === undefined ? { token: name } : { token: name, alpha });

/**
 * The tokens of the frame being built. The theme and viewport are read
 * from the host each call (kit parses them only when they change), so two
 * engines at the same tick never share a stale surface; what is derived
 * from them is kept while they are the same objects.
 */
type Frame = {
  theme: Api.UiTheme; viewport: Api.Viewport; t: Tokens; phone: boolean; touch: boolean;
  /**
   * A version-1 style (UiTheme.styleVersion): a game shipped with it keeps
   * the screens it shipped with, so a template lays its screen out as it
   * did then and adds no rows, rules or marks since (docs/ui.md §12.1).
   */
  v1: boolean;
};
let cached: Frame | null = null;
export function frame(): Frame {
  const theme = kit.frameTheme();
  const viewport = kit.frameViewport();
  if (cached && cached.theme === theme && cached.viewport === viewport) return cached;
  const phone = viewport.breakpoint === "phonePortrait" || viewport.breakpoint === "phoneLandscape";
  cached = { theme, viewport, t: tokens(theme), phone, touch: viewport.coarsePointer || phone || viewport.breakpoint === "tablet", v1: theme.styleVersion === 1 };
  return cached;
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
