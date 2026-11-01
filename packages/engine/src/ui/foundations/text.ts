// Foundations (docs/ui.md §11): text in a role, prose, the separator and
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

export function prose(id: string, markup: string, desc: Api.ProseDesc = {}): Api.RichTextResult {
  const { t } = frame();
  const rt = roleType(desc.role ?? "body");
  const paras = markup.split(/\n[ \t]*\n+/).map((p) => p.trim()).filter((p) => p.length > 0);
  const width = desc.width ?? grow(undefined, (desc.measure ?? true) ? measure(rt.step) : undefined);
  let out: Api.RichTextResult = { response: { hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, adjust: 0 } as Api.Response, link: null, hovered: null };
  kit.open(id, { direction: "column", width, gap: desc.gap ?? t.md, cross: "stretch" });
  try {
    paras.forEach((p, i) => {
      const r = kit.richText(kit.idi("P", i), p, { step: rt.step, color: token(toneColor(desc.tone)), align: desc.align ?? "start", width: grow(), font: rt.font }) as Api.RichTextResult;
      if (r.link) out = { ...out, link: r.link, response: r.response };
      if (r.hovered) out = { ...out, hovered: r.hovered };
    });
  } finally {
    kit.close();
  }
  kit.semantics({ role: "group" });
  return out;
}

export function separator(id: string, desc: Api.SeparatorDesc = {}): void {
  const { t } = frame();
  const vertical = desc.vertical ?? false;
  const inset = desc.inset ?? 0;
  const ink: Api.ThemeColor = desc.strong ? "lineStrong" : "line";
  const line = (lid: string, grows: boolean): void => {
    kit.open(lid, {
      width: vertical ? px(1) : grows ? grow() : px(0),
      height: vertical ? grow() : px(1),
      background: token(ink),
      part: `Separator.line${desc.strong ? "[strong]" : ""}`,
      partClass: "bar",
    });
    kit.close();
  };
  if (!desc.label && !desc.strong && !vertical) {
    // The style's own divider (its ornament where it has one), padded at its ends.
    kit.open(id, { direction: "row", width: grow(), padding: { left: inset, right: inset, top: 0, bottom: 0 }, cross: "center" });
    kit.divider("Line", false);
    kit.close();
    kit.semantics({ role: "separator" });
    return;
  }
  kit.open(id, {
    direction: vertical ? "column" : "row",
    width: vertical ? px(1) : grow(),
    height: vertical ? grow() : null,
    padding: vertical ? { left: 0, right: 0, top: inset, bottom: inset } : { left: inset, right: inset, top: 0, bottom: 0 },
    gap: t.sm,
    cross: "center",
    align: "center",
  });
  try {
    line("Start", true);
    if (desc.label && !vertical) {
      label("Label", desc.label, { step: "caption", color: "inkMuted", nowrap: true });
      line("End", true);
    }
  } finally {
    kit.close();
  }
  kit.semantics({ role: "separator", ...(desc.label ? { name: desc.label } : {}) });
}

export function space(id: string, desc: Api.SpaceDesc = {}): void {
  if (desc.grow) {
    kit.spacer(id, grow());
    return;
  }
  const s = frame().theme.space[desc.step ?? "md"] ?? 16;
  kit.spacer(id, px(s));
}

