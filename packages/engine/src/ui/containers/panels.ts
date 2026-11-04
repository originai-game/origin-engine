// Panels and cards, the scroll area, the accordion and split views
// (docs/ui.md §11, Containers). Frames are parts a style dresses
// (`Card.frame[selected]`, `Accordion.header[expanded]`, `Split.divider`),
// drawn from the tokens (the element class's shape, stroke and elevation)
// where it does not; only decoration (chevrons, rings, fades) is drawn
// with og.ui.setDraw.

import type * as Api from "@origingame/engine";
import {
  all, byText, chevron, clamp, fit, flags, over, frame, grow, kit, lastGesture, lastInteraction, px, rememberPath, role, rtl, spring, still, surface, text, token, xy,
} from "./base.ts";

// ── card ────────────────────────────────────────────────────────────────

const CARD_CLASS: Record<string, string> = { raised: "card", flat: "panel", outlined: "panel", inset: "slot" };
const CARD_FILL: Record<string, Api.ThemeColor> = { raised: "panelRaised", flat: "panel", outlined: "panel", inset: "well" };

/**
 * A card: a framed surface with an optional media slot (art the game
 * fills, or a placeholder drawn from the style), an eyebrow, a title, a
 * subtitle and a corner badge over a body the game builds. Interactive
 * cards lift under the pointer and the focus, sink when pressed, and take
 * keys and pads; a selected card is ringed in the focus ink.
 */
export function card(id: string, desc: Api.CardDesc, build: Api.UiBuild): Api.Response {
  const { t } = frame();
  const variant = desc.variant ?? "raised";
  const selected = desc.selected ?? false;
  const disabled = desc.disabled ?? false;
  const fillRole = CARD_FILL[variant] ?? "panelRaised";
  const pad = desc.padding ?? t.lg;
  const line = (step: string, px0: number): number => frame().theme.typeRamp[step]?.lineHeight ?? byText(px0);
  // The corner marks: small beside the art (a number plate a caption's line tall, a cost disc a large numeral's).
  const plateH = line("sm", 20) + 4;
  const disc = Math.max(byText(28), line("lg", 26) + 6);
  const content = (): void => {
    if (desc.eyebrow) text("Eyebrow", desc.eyebrow, { step: "caption", color: "inkMuted", width: grow() });
    if (desc.title) text("Title", desc.title, { step: "lg", heading: true, width: grow() });
    if (desc.subtitle) text("Subtitle", desc.subtitle, { step: "sm", color: "inkMuted", width: grow() });
    if (desc.title || desc.subtitle || desc.eyebrow) kit.spacer("Gap", px(t.xs));
    build();
  };
  const body = (): void => {
    // With art the cost disc sits on the art's bottom end corner (the number plate's opposite), so the body's
    // text keeps the card's whole width: a narrow card in a script without spaces wrapped its effect short of it.
    const costDisc = (): void => {
      kit.open("Cost", { direction: "column", width: px(disc), height: px(disc), align: "center", cross: "center", part: "Card.cost", partClass: "badge" });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("well") }], strokes: [{ width: 1.5, align: "inside", paint: { kind: "solid", color: token("lineStrong") } }] });
      text("Text", desc.cost ?? "", { step: "lg", font: "rounded", nowrap: true });
      kit.close();
      over({ insets: { bottom: t.sm, end: t.sm }, z: 2 });
    };
    if (desc.media) media("Media", desc.media, t.panel, !!desc.badge, desc.cost ? costDisc : undefined);
    // With no media the number plate stands over the body's top: the body starts under it.
    const padding = desc.number && !desc.media ? { ...all(pad), top: t.sm + plateH + t.xs } : all(pad);
    if (desc.cost && !desc.media) {
      // With no art the cost disc beside the body at its bottom end, so the game's text wraps short of it.
      kit.row("Body", { width: grow(), padding, gap: t.sm, cross: "end" }, () => {
        kit.column("Content", { width: grow(), gap: t.xs }, content);
        kit.open("Cost", { direction: "column", width: px(disc), height: px(disc), align: "center", cross: "center", part: "Card.cost", partClass: "badge" });
        kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("well") }], strokes: [{ width: 1.5, align: "inside", paint: { kind: "solid", color: token("lineStrong") } }] });
        text("Text", desc.cost ?? "", { step: "lg", font: "rounded", nowrap: true });
        kit.close();
      });
    } else kit.column("Body", { width: grow(), padding, gap: t.xs }, content);
    if (desc.badge) {
      kit.badge("Badge", desc.badge, desc.badgeTone ?? "primary");
      over({ insets: { top: t.sm, end: t.sm }, z: 2 });
    }
    if (desc.number) {
      kit.open("Number", { direction: "row", width: fit(plateH, plateH * 3), height: px(plateH), align: "center", cross: "center", padding: xy(t.xs, 0), part: "Card.number", partClass: "badge" });
      kit.setDraw({ shape: { kind: "rect", radii: [4, 4, 4, 4] }, fills: [{ kind: "solid", color: token("panelRaised", 0.92) }], strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: token("lineStrong") } }] });
      text("Text", desc.number, { step: "sm", font: "display", nowrap: true });
      kit.close();
      over({ insets: { top: t.sm, start: t.sm }, z: 2 });
    }
    if (selected) ring("Selected");
  };
  const part = `Card.frame${flags(selected && "selected", variant)}`;
  // A fixed width is the card at 100 % text: it grows with the player's text size, as far as the room it
  // is given (a card in a slot of its own 100 % width stays in the slot, its words wrapping inside it).
  const base = desc.width?.kind === "fixed" ? desc.width.px ?? 0 : 0;
  const width = base > 0 ? grow(base, Math.max(base, byText(base))) : desc.width ?? null;
  if (desc.interactive) {
    const r = surface(id, {
      part: "Card.frame",
      cls: CARD_CLASS[variant] ?? "card",
      flags: [selected ? "selected" : "", variant],
      selected,
      fill: role(fillRole),
      washBy: 0.07,
      // Lifts under the pointer and the focus, sinks while pressed: the style's ui spring.
      lift: -4,
      pressScale: 0.95,
      disabled,
      direction: "column",
      width,
      height: desc.height ?? null,
      cross: "stretch",
      radius: t.panel,
      semantics: { role: "button", selected, ...(desc.title ? { name: [desc.eyebrow, desc.title, desc.subtitle].filter(Boolean).join(", ") } : {}) },
      // Over its neighbours while it lifts.
      props: { z: lastInteraction(id).hovered || lastInteraction(id).focused ? 1 : 0 },
    }, body);
    return r;
  }
  kit.open(id, {
    direction: "column",
    width,
    height: desc.height ?? null,
    cross: "stretch",
    radius: t.panel,
    background: token(fillRole),
    part,
    partClass: CARD_CLASS[variant] ?? "card",
  });
  let r: Api.Response;
  try {
    body();
  } finally {
    r = kit.close() as Api.Response;
  }
  kit.semantics({ role: "group", ...(desc.title ? { name: desc.title } : {}) });
  return r;
}

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

/** A media slot: the game's texture covering the slot at its aspect around its focal point, or a placeholder drawn from the style and the slot's kind (never a grey box). */
export function media(id: string, m: Api.CardMedia, radius: number, badged = false, corner?: () => void): void {
  const aspect = m.aspect && m.aspect > 0 ? m.aspect : 16 / 9;
  // Under a corner badge the placeholder's mark sits low in the slot, clear of the badge.
  const low = badged && !m.texture;
  kit.open(id, { direction: "column", width: grow(), aspect, align: low ? "end" : "center", cross: "center", padding: low ? { left: 0, top: 0, right: 0, bottom: frame().t.md } : all(0), part: `Card.media${flags(!m.texture && "placeholder")}`, partClass: "slot" });
  const shape: Api.DrawShape = { kind: "rect", radii: [radius, radius, 0, 0] };
  if (m.texture) {
    kit.setDraw({ shape, fills: [{ kind: "image", texture: m.texture, fit: "cover", focal: m.focal ?? { x: 0.5, y: 0.5 } }] });
  } else {
    kit.setDraw({
      shape,
      fills: [
        { kind: "gradient", type: "linear", from: { x: 0, y: 0 }, to: { x: 1, y: 1 }, stops: [{ offset: 0, color: token("primarySoft") }, { offset: 1, color: token("well") }] },
        { kind: "gradient", type: "radial", from: { x: 0.5, y: 0.4 }, to: { x: 0.65, y: 0.65 }, stops: [{ offset: 0, color: token("sheen", 0.45) }, { offset: 1, color: token("sheen", 0) }] },
      ],
    });
    kit.icon("Kind", m.icon || "image", { size: 32, tint: token("inkFaint"), label: "" });
  }
  // A mark the card pins to the art (its cost disc), laid out with it.
  corner?.();
  kit.close();
  if (m.label) kit.semantics({ role: "image", name: m.label });
}

// ── scroll area ─────────────────────────────────────────────────────────

/**
 * A scroll area: og.ui's scroll container (the wheel, a drag with
 * momentum, two fingers, draggable bars, focus scrolled into view, nested
 * handoff) whose content fades into the surface it sits on at an edge
 * where more of it lies beyond, so a cut list reads as "there is more".
 */
export function scrollArea(id: string, desc: Api.ScrollAreaDesc, build: Api.UiBuild): void {
  const { t } = frame();
  const horizontal = desc.axes === "horizontal";
  const fadeRole: Api.ThemeColor = desc.fadeInto ?? "panel";
  const fade = desc.fade && desc.fade > 0 ? desc.fade : 24;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), height: desc.height ?? null, part: "ScrollArea.frame", partClass: "panel" });
  try {
    kit.openScroll("View", { width: grow(), height: desc.height ? grow() : null, padding: all(0), gap: 0, axes: desc.axes ?? "vertical" });
    try {
      kit.spacer("Start", px(0));
      kit.open("Content", { direction: horizontal ? "row" : "column", width: horizontal ? null : grow(), padding: desc.padding ?? all(0), gap: desc.gap ?? t.sm, cross: horizontal ? "start" : "stretch" });
      try {
        build();
      } finally {
        kit.close();
      }
      kit.spacer("End", px(0));
    } finally {
      kit.close();
    }
    const view = kit.lastPath();
    // Whether the content's ends are in view, last tick: a marker scrolled out of view is not listed.
    const vb = kit.bounds(view);
    const first = kit.bounds(`${view}/Start`);
    const last = kit.bounds(`${view}/End`);
    const before = vb !== null && (first === null || (horizontal ? first.x < vb.x - 0.5 : first.y < vb.y - 0.5));
    const after = vb !== null && (last === null || (horizontal ? last.x > vb.x + vb.width + 0.5 : last.y > vb.y + vb.height + 0.5));
    edge("FadeStart", true, kit.animate("~start", before ? 1 : 0, spring("ui")), horizontal, fade, fadeRole);
    edge("FadeEnd", false, kit.animate("~end", after ? 1 : 0, spring("ui")), horizontal, fade, fadeRole);
  } finally {
    kit.close();
  }
  kit.semantics({ role: "region", ...(desc.label ? { name: desc.label } : {}) });
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
          exit: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: -6 }], transition: { kind: "tween", durationMs: 120, ease: "exit" } },
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

// ── split view ──────────────────────────────────────────────────────────

/**
 * Two panes side by side (or one over the other) with a divider between
 * them. A drag moves the divider with the pointer and the panes take the
 * new ratio on release (on the panel spring); a press of Enter or the
 * pad's A on the focused divider steps through the snap ratios. Each pane
 * keeps its minimum. On a phone in portrait a side-by-side split stacks
 * unless `keepOnPhone`. Answers the ratio to keep in game state.
 */
export function splitView(id: string, desc: Api.SplitViewDesc, start: Api.UiBuild, end: Api.UiBuild): Api.SplitResult {
  const { viewport } = frame();
  const stacked = !desc.keepOnPhone && viewport.breakpoint === "phonePortrait" && (desc.direction ?? "row") === "row";
  const dir: Api.Direction = stacked ? "column" : desc.direction ?? "row";
  const across = dir === "row";
  const ratio0 = clamp(desc.ratio ?? 0.5, 0, 1);
  const handle = 12;
  const key = `split:${id}`;
  const size = kit.memory(`${id}~size`, 0);
  kit.open(id, { direction: dir, width: desc.width ?? grow(), height: desc.height ?? grow(), cross: "stretch" });
  let ratio = ratio0;
  let changed = false;
  try {
    const total = Math.max(1, size - handle);
    const lo = Math.min(0.5, (desc.minStart ?? 120) / total);
    const hi = Math.max(0.5, 1 - (desc.minEnd ?? 120) / total);
    // The divider's drag, last tick: its offset is the ratio it is dragged to; a release commits it.
    const ges = lastGesture(`${key}/Divider`);
    const held = kit.memory("~held", 0);
    const dragged = kit.memory("~drag", 0);
    if (ges && !ges.dragging && held !== 0) {
      ratio = clamp(ratio0 + dragged / total, lo, hi);
      changed = Math.abs(ratio - ratio0) > 1e-4;
      if (changed) kit.feedback("select", -1);
    }
    kit.remember("~held", ges?.dragging ? 1 : 0);
    kit.remember("~drag", ges?.dragging ? (across ? ges.offsetX * (kit.frameDirection() === "rtl" ? -1 : 1) : ges.offsetY) : 0);
    const shown = stacked ? 0.5 : ratio;
    pane("Start", across, size > 0 ? Math.round(shown * total) : 0, start);
    const r = surface("Divider", {
      part: "Split.divider",
      cls: "bar",
      fill: null,
      washBy: 0.16,
      width: across ? px(handle) : grow(),
      height: across ? grow() : px(handle),
      align: "center",
      cross: "center",
      radius: handle / 2,
      semantics: { role: "separator", name: desc.label ?? "", valueMin: 0, valueMax: 100, valueNow: Math.round(ratio * 100), value: `${Math.round(ratio * 100)}%` },
      props: { drag: { axis: across ? "x" : "y", returnHome: true, threshold: 3, spring: { kind: "instant" } }, hitSlop: 6 },
    }, () => {
      const was = lastInteraction("Divider");
      const hot = was.hovered || was.pressed || was.focused || (ges?.held ?? false);
      kit.open("Grip", { width: across ? px(hot ? 4 : 2) : px(40), height: across ? px(40) : px(hot ? 4 : 2), radius: 2, background: token(hot ? "focus" : "lineStrong") });
      kit.close();
      kit.props({ motion: spring("ui") });
    });
    rememberPath(`${key}/Divider`, kit.lastPath());
    if (r.clicked && !ges?.dragging && held === 0) {
      const snaps = desc.snaps && desc.snaps.length > 0 ? desc.snaps : [0.33, 0.5, 0.67];
      ratio = clamp(snaps.find((s) => s > ratio0 + 0.01) ?? snaps[0]!, lo, hi);
      changed = true;
      kit.feedback("select", -1);
    }
    pane("End", across, 0, end);
  } finally {
    kit.close();
  }
  const b = kit.bounds(kit.lastPath());
  kit.remember(`${id}~size`, b ? (across ? b.width : b.height) : size);
  return { ratio, changed, stacked };
}

/** A pane `length` CSS px along the split (0: it takes what is left). */
function pane(id: string, across: boolean, length: number, build: Api.UiBuild): void {
  const along: Api.Size = length > 0 ? px(length) : grow();
  kit.open(id, {
    direction: "column",
    width: across ? along : grow(),
    height: across ? grow() : along,
    cross: "stretch",
    part: "Split.pane",
    partClass: "panel",
  });
  try {
    build();
  } finally {
    kit.close();
  }
  kit.props({ layoutMotion: spring("panel") });
  kit.semantics({ role: "region" });
}
