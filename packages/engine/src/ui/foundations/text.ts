// Foundations (docs/ui.md §2): text in a role, prose, the separator and
// space by the style's scale. A text's role picks its type step and font
// from the style, so a style's typography (and the player's text size)
// moves every text at once; a tone picks its ink by meaning.

import type * as Api from "@origingame/engine";
import { frame, grow, kit, px, text as label, token } from "../controls/base.ts";

/** A role's type step by breakpoint (a phone's display is a step smaller), its font and whether it is a heading. */
export function roleType(r: Api.TextRole | undefined): { step: Api.TypeStepName; font: Api.FontRole; heading: boolean } {
  const phone = frame().phone;
  switch (r ?? "body") {
    case "display": return { step: phone ? "xl4" : "displaySm", font: "display", heading: true };
    case "title": return { step: phone ? "xl2" : "xl3", font: "display", heading: true };
    case "heading": return { step: phone ? "lg" : "xl", font: "body", heading: true };
    case "label": return { step: "sm", font: "body", heading: false };
    case "caption": return { step: "caption", font: "body", heading: false };
    case "numeric": return { step: "body", font: "mono", heading: false };
    default: return { step: "body", font: "body", heading: false };
  }
}

const TONE: Record<Api.TextTone, Api.ThemeColor> = {
  normal: "ink",
  muted: "inkMuted",
  faint: "inkFaint",
  primary: "primary",
  success: "success",
  warn: "warn",
  danger: "danger",
  info: "info",
  inverse: "inkInverse",
};

/** A tone's ink as a theme colour. */
export function toneColor(t: Api.TextTone | undefined): Api.ThemeColor {
  return TONE[t ?? "normal"] ?? "ink";
}

export function text(id: string, value: string, desc: Api.TextBlockDesc = {}): void {
  const rt = roleType(desc.role);
  const max = desc.maxLines ?? 0;
  kit.label(id, value, {
    step: desc.step ?? rt.step,
    color: token(toneColor(desc.tone)),
    align: desc.align ?? "start",
    nowrap: desc.nowrap ?? false,
    ellipsis: max > 0 || (desc.nowrap ?? false),
    font: rt.font,
    heading: rt.heading,
    ...(desc.width ? { width: desc.width } : {}),
    ...(max > 0 ? { maxLines: max } : {}),
  });
}

/** About 66 characters of a type step: half an em a character, as body text averages (the ramp carries the player's text size). */
function measure(step: Api.TypeStepName): number {
  return Math.round((frame().theme.typeRamp[step]?.size ?? 16) * 33);
}

export function space(id: string, desc: Api.SpaceDesc = {}): void {
  if (desc.grow) {
    kit.spacer(id, grow());
    return;
  }
  const s = frame().theme.space[desc.step ?? "md"] ?? 16;
  kit.spacer(id, px(s));
}

