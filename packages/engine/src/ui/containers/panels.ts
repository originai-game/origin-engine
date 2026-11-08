// Panels and cards, the scroll area and the accordion (docs/ui.md §2,
// Containers; the split view is a component document in
// src/ui/components). Frames are parts a style dresses
// (`Card.frame[selected]`, `Accordion.header[expanded]`), drawn from the
// tokens (the element class's shape, stroke and elevation)
// where it does not; only decoration (chevrons, rings, fades) is drawn
// with og.ui.setDraw.

import type * as Api from "@origingame/engine";
import {
  chevron, fit, flags, over, frame, grow, kit, px, role, rtl, spring, surface, text, token, xy,
} from "./base.ts";

// ── card ────────────────────────────────────────────────────────────────

/**
 * A ring just inside an element's edge in the focus ink (a selected card or cell), drawn over its content, taking no
 * input; with a `part`, a style can dress it (the default style's borderless selection draws none).
 */
export function ring(id: string, width = 2, ink: Api.ThemeColor = "focus", part?: string): void {
  kit.open(id, { width: grow(), height: grow(), ...(part ? { part } : {}) });
  kit.setDraw({ shape: { kind: "rect" }, strokes: [{ width, align: "inside", paint: { kind: "solid", color: token(ink) } }] });
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 3 });
}

/** A fade over a scrolling view's edge (`k` its strength, 0 to 1) into the surface it sits on: more lies that way. */
export function edge(id: string, start: boolean, k: number, horizontal: boolean, size: number, ink: Api.ThemeColor): void {
  if (k < 0.01) return;
  kit.open(id, { width: horizontal ? px(size) : grow(), height: horizontal ? grow() : px(size) });
  kit.setDraw({
    fills: [{
      kind: "gradient",
      type: "linear",
      // Opaque at the outer side: the start inset is the right under a right-to-left locale.
      from: horizontal ? { x: start !== rtl() ? 0 : 1, y: 0.5 } : { x: 0.5, y: start ? 0 : 1 },
      to: horizontal ? { x: start !== rtl() ? 1 : 0, y: 0.5 } : { x: 0.5, y: start ? 1 : 0 },
      stops: [{ offset: 0, color: token(ink, k) }, { offset: 1, color: token(ink, 0) }],
    }],
  });
  kit.close();
  over({
    insets: horizontal ? (start ? { start: 0, top: 0, bottom: 0 } : { end: 0, top: 0, bottom: 0 }) : start ? { start: 0, end: 0, top: 0 } : { start: 0, end: 0, bottom: 0 },
    inert: true,
    z: 3,
  });
}

// ── accordion ───────────────────────────────────────────────────────────

/**
 * An accordion: sections whose headers open and close their bodies. A
 * body enters fading down into place and leaves fading up, the sections
 * under it move on the panel spring (layout animation) rather than jump,
 * and the chevron turns. With `single` opening one closes the others.
 * Each header is a button that says expanded or collapsed; its body is a
 * region named by its title.
 */
export function accordion(id: string, desc: Api.AccordionDesc, build: Api.UiItemBuild): Api.AccordionResult {
  const { t } = frame();
  const sections = desc.sections ?? [];
  let expanded = [...(desc.expanded ?? [])];
  let toggled: number | null = null;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), gap: desc.gap ?? 0, cross: "stretch" });
  try {
    sections.forEach((s, i) => {
      const open = expanded.includes(i);
      kit.open(kit.idi("Section", i), { direction: "column", width: grow(), cross: "stretch", part: `Accordion.section${flags(open ? "expanded" : "collapsed")}`, partClass: "panel" });
      const r = surface("Header", {
        part: "Accordion.header",
        cls: "tab",
        flags: [open ? "expanded" : "collapsed"],
        expanded: open,
        fill: null,
        disabled: s.disabled ?? false,
        width: grow(),
        height: fit(Math.max(t.controlLg, 44)),
        padding: xy(t.md, t.sm),
        gap: t.md,
        semantics: { role: "button", expanded: open, name: s.subtitle ? `${s.title ?? ""}, ${s.subtitle}` : s.title ?? "" },
      }, () => {
        kit.column("Text", { width: grow(), gap: 2 }, () => {
          text("Title", s.title ?? "", { step: "base", heading: true, width: grow() });
          if (s.subtitle) text("Subtitle", s.subtitle, { step: "caption", color: "inkMuted", width: grow() });
        });
        if (s.badge) kit.badge("Badge", s.badge, s.badgeTone ?? "neutral");
        chevron("Chevron", "down", 18, role(s.disabled ? "inkFaint" : "inkMuted"), { group: "expansion", current: open ? "expanded" : "collapsed", turns: { collapsed: 0, expanded: 2 } });
      });
      if (r.clicked) {
        toggled = i;
        kit.feedback(open ? "close" : "open", -1);
        expanded = open ? expanded.filter((k) => k !== i) : desc.single ? [i] : [...expanded, i];
      }
      if (open) {
        kit.column("Body", { width: grow(), padding: { left: t.md, top: 0, right: t.md, bottom: t.md }, gap: t.sm, cross: "stretch" }, () => build(i));
        kit.semantics({ role: "region", name: s.title ?? "" });
        kit.props({
          enter: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: -8 }], transition: spring("panel") },
          exit: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: -6 }], transition: { kind: "tween", durationMs: 120, bezier: [0.4, 0, 1, 1] } },
          reduced: "fade",
        });
      }
      if (i < sections.length - 1 && desc.dividers !== false) kit.divider("Divider", false);
      kit.close();
      kit.props({ layoutMotion: spring("panel") });
    });
  } finally {
    kit.close();
  }
  return { toggled, expanded };
}
