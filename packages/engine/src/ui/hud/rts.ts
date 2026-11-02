// og.ui.selectionPanel, og.ui.commandCard, og.ui.productionQueue,
// og.ui.rtsConsole and og.ui.dragSelect: the RTS rows of the genre sheets
// (B-rts-hud). Read against StarCraft II's and Age of Empires IV's
// consoles: one continuous panel along the bottom, divided into zones by
// hairlines (the minimap, the selection under its control group tabs, the
// focused unit's card, the command card); the selection a grid of
// portraits each with a thin health bar under its art, the focused one in
// the selected-tile rule; the unit card's portrait at full height beside
// its name, type, health numeral over a bar and stats in tabular figures;
// the command card a fixed grid whose cells print their hotkey small in a
// corner, empty cells the panel's well; a building's queue a row of small
// slots with the first one's progress sweeping; and the drag-selection box
// with a faint fill, a bright edge and corner ticks. The pack dresses it:
// nothing here draws a glow edge or a bevel of a particular look.

import type * as Api from "@origingame/engine";
import { BLACK, WHITE, a11y, absolute, anchorOf, arc, artTile, box, bp, chip, clamp01, close, countdown, edge, fillDraw, frame, glow, groups, innerGlow, hudPlace, hudRoom, hueOf, icon, keyId, kit, leaf, lit, mix, open, pct, px, role, sel, shapeOf, solid, sound, stepSize, stroke, text, token, ts, tsFit, u, ut, vgrad, type Rgba } from "./core.ts";
import { slot } from "./abilities.ts";
import { minimapView } from "./nav.ts";

/** The console's ground: the HUD chip denser, over a blurred scene, so its zones sit on an even ground. */
function panelDraw(): Api.DrawDesc {
  const c = chip();
  return { ...c, fills: [solid(token("hud", 0.9)), ...(c.fills ?? []).slice(1)], effects: [{ kind: "backdrop", blur: u(12) }, ...(c.effects ?? [])] };
}

/** A hairline between two zones of one panel. */
function rule(id: string, h: number): void {
  leaf(id, { width: px(1), height: px(h), part: sel("RtsConsole", "rule") }, { fills: [solid(token("line", 0.9))] });
}

/** Health one muted hue falling green, amber, red (the raid frames' ramp), so a hurt unit reads in a grid at a glance. */
const GREEN: Rgba = { r: 0.27, g: 0.66, b: 0.36, a: 1 };
const AMBER: Rgba = { r: 0.86, g: 0.62, b: 0.18, a: 1 };
const RED: Rgba = { r: 0.82, g: 0.26, b: 0.22, a: 1 };
function healthInk(f: number): Rgba {
  const ramp = clamp01((0.6 - f) / 0.45);
  return ramp <= 0 ? GREEN : ramp < 0.5 ? mix(GREEN, AMBER, ramp * 2) : mix(AMBER, RED, (ramp - 0.5) * 2);
}
const SHIELD: Rgba = { r: 0.45, g: 0.72, b: 1, a: 1 };
const ENERGY: Rgba = { r: 0.7, g: 0.5, b: 1, a: 1 };

/** A result built up over a tick. */
type Mut<T> = { -readonly [K in keyof T]: T[K] };

function hash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619) >>> 0;
  return h % 1e9;
}

/** The well an empty cell or slot shows: the panel's own recess. */
function well(r: number): Api.DrawDesc {
  return {
    shape: { kind: "rect", radii: [r, r, r, r] },
    fills: [solid(token("well", 0.55))],
    strokes: [stroke(1, token("line", 0.5))],
  };
}

// ── production queue ──────────────────────────────────────────────────

/** The time left on the item being built as its slot prints it: whole seconds under a minute, else m:ss; or paused. */
function queueTime(desc: Api.ProductionQueueDesc, it: Api.QueueItem): string {
  if (desc.paused) return desc.pausedLabel ?? "Paused";
  return (it.remaining ?? 0) < 60 ? String(Math.ceil(it.remaining ?? 0)) : countdown(it.remaining ?? 0, false);
}

/**
 * The queue's row, in place (no panel): the first item with its progress sweeping and seconds left, then the rest,
 * then wells. `from` and `to` draw a range of the slots (a building's zone draws the first one large, the rest after it).
 */
function queueRow(id: string, desc: Api.ProductionQueueDesc, side: number, from = 0, to = Infinity, timeInside = true): Api.ProductionQueueResult {
  const items = desc.items ?? [];
  const slots = Math.min(to, Math.max(items.length, desc.slots || 5));
  const gap = Math.max(2, u(3));
  const r = Math.min(shapeOf("slot").radius, Math.max(3, side / 8));
  const paused = !!desc.paused;
  let cancelled: string | null = null;
  const first = items[0];
  if (from === 0) {
    const e = edge(`${id}~first`, first ? hash(first.key ?? first.name ?? "") : -1);
    sound("success", e.changed && e.prev !== -1 && kit.memory(`${id}~seen`, 0) > 0);
    kit.remember(`${id}~seen`, 1);
  }
  const state = items.length === 0 ? "idle" : paused ? "paused" : "building";
  box(id, { direction: "row", gap, cross: "center", part: sel("ProductionQueue", "frame"), partClass: "hud" }, undefined, () => {
    for (let i = from; i < slots; i++) {
      const it = items[i];
      if (!it) {
        leaf(kit.idi("Empty", i), { width: px(side), height: px(side), part: sel("ProductionQueue", "empty") }, well(r));
        continue;
      }
      const k = keyId(it.key ?? it.name ?? String(i));
      open(k, { width: px(side), height: px(side), interactive: true, part: sel("ProductionQueue", i === 0 ? "current" : "item") }, {
        shape: { kind: "rect", radii: [r, r, r, r] },
        fills: [solid(lit(BLACK, 0.9))],
        strokes: [stroke(1, token("lineStrong", 0.45), "outside")],
        clip: "shape",
      });
      artTile("Art", it.image, it.icon ?? "", side, hueOf(it.icon || it.name || k), absolute({ start: 0, top: 0 }));
      if (i === 0) {
        const dur = Math.max(1e-3, it.duration ?? 0);
        const left = (it.duration ?? 0) > 0 ? clamp01((it.remaining ?? 0) / dur) : 1;
        // The part still to build darkened clockwise from the top, as an ability's cooldown: the same sweep grammar.
        if (left > 0) {
          leaf("Sweep", { width: px(side * 1.5), height: px(side * 1.5), part: sel("ProductionQueue", "bar", paused ? "paused" : null) }, {
            shape: { kind: "arc", start: -Math.PI / 2 + (1 - left) * Math.PI * 2, sweep: Math.max(0.0001, left) * Math.PI * 2, thickness: 4096, cap: "butt" },
            fills: [solid(lit(BLACK, 0.62))],
          }, absolute({ start: -side * 0.25, top: -side * 0.25 }));
        }
        if (timeInside) {
          text("Time", queueTime(desc, it), {
            step: tsFit(11, side * 0.32, true), font: "mono", color: paused ? lit(role("warn")) : token("ink"), nowrap: true, outline: 1,
          }, absolute({ centerX: true, centerY: true }));
        }
      }
      if ((it.count ?? 0) > 1) {
        text("Count", `${it.count}`, { step: tsFit(9, side * 0.3, true), font: "display", color: token("ink"), nowrap: true, outline: 1 }, absolute({ end: Math.max(2, u(2)), bottom: 0 }));
      }
      const res = close({ a11y: a11y("button", it.name ?? "", i === 0 ? { value: paused ? (desc.pausedLabel ?? "Paused") : `${Math.ceil(it.remaining ?? 0)}s` } : {}) });
      kit.tooltip(it.name ?? "");
      if (res.clicked) cancelled = it.key ?? "";
    }
  }, {
    a11y: a11y("list", "Production queue"),
    ...groups({ queue: state }, { queue: ["idle", "building", "paused"] }),
  });
  return { cancelled };
}

export function productionQueue(id: string, desc: Api.ProductionQueueDesc = {}): Api.ProductionQueueResult {
  const side = u(desc.size || (bp().phone ? 36 : 40));
  let res: Api.ProductionQueueResult = { cancelled: null };
  const anchor = anchorOf(desc, null);
  hudPlace(id, anchor, (inAnchor) => {
    if (!inAnchor) {
      res = queueRow(id, desc, side);
      return;
    }
    box(`${id}Panel`, { padding: { left: frame().t.sm, right: frame().t.sm, top: frame().t.sm, bottom: frame().t.sm } }, panelDraw(), () => {
      res = queueRow(id, desc, side);
    });
  });
  return res;
}

// ── selection ─────────────────────────────────────────────────────────

/** A tab's height: the label's at the text size, but never more than a fifth of the zone it stands on (200 % text). */
function tabHeight(zone: number): number {
  return Math.max(u(20), Math.min(ut(26), Math.round(zone * 0.22)));
}

/**
 * The control group tabs, seated on the panel's top edge as tabs of the selection zone: the current one on the panel's
 * own ground (a seam over the panel's top hairline lets that ground run on into it), lit by the highlight along its top;
 * the others a step back on a darker ground. Labels at the caption step.
 */
function tabs(desc: Api.SelectionPanelDesc, tabH: number, onClick: (k: string) => void, panelPath = ""): void {
  const list = desc.groups ?? [];
  if (list.length === 0) return;
  const r = Math.min(shapeOf("hud").radius, u(6));
  const capStep: Api.TypeStepName = stepSize("caption").size <= tabH * 0.62 ? "caption" : tsFit(12, tabH * 0.62, true);
  kit.row("Tabs", { gap: Math.max(2, u(2)), cross: "end" }, () => {
    for (const g of list) {
      const cur = (g.key ?? "") === (desc.group ?? "");
      const empty = (g.count ?? 0) === 0;
      open(keyId(g.key ?? g.label ?? ""), {
        direction: "row", gap: 0, cross: "center", align: "center", interactive: true,
        height: px(tabH), width: { kind: "fit", min: Math.round(tabH * 1.5) }, padding: { left: u(14), right: u(14), top: 0, bottom: 0 },
        part: sel("SelectionPanel", "tab", cur ? "current" : null, empty ? "empty" : null),
      }, cur ? {
        // The panel's ground and edge (panelDraw), so the current tab and the panel read as one surface.
        shape: { kind: "rect", radii: [r, r, 0, 0] },
        fills: [solid(token("hud", 0.9))],
        strokes: [stroke(1, token("lineStrong", 0.55))],
        effects: [{ kind: "backdrop", blur: u(12) }],
      } : {
        shape: { kind: "rect", radii: [r, r, 0, 0] },
        fills: [solid(token("hud", 0.72)), solid(lit(BLACK, 0.18))],
        strokes: [stroke(1, token("line", 0.6))],
      });
      if (cur) {
        leaf("Lit", { width: pct(1), height: px(Math.max(2, u(2))), part: sel("SelectionPanel", "tabLit") }, { shape: { kind: "rect", radii: [r, r, 0, 0] }, fills: [solid(token("highlight"))] }, absolute({ start: 0, top: 0 }));
        // Over the tab's foot and the panel's top hairline under it: the panel's ground running on into the tab.
        if (panelPath) leaf("Seam", { width: pct(1), height: px(2), part: sel("SelectionPanel", "seam") }, { fills: [solid(token("hud", 0.9))] }, absolute({ start: 0, bottom: -1 }, { z: 2, overlaps: [panelPath] }));
      }
      text("Label", g.label ?? "", { step: capStep, font: "display", color: cur ? token("ink") : token("inkMuted"), nowrap: true });
      const res = close({ opacity: empty ? 0.55 : 1, a11y: a11y("tab", g.label ?? "", { value: `${g.count ?? 0}`, selected: cur }) });
      if (res.clicked) onClick(g.key ?? "");
    }
  });
  // Over the panel built after them, so the current tab's seam lies over the panel's top hairline.
  if (panelPath) kit.props({ z: 1 });
}

/** One unit's tile: its portrait, a thin health bar under the art (its shield and energy beside it), a gold build line. */
function tile(u0: Api.SelectedUnit, side: number, focused: boolean): Api.Response {
  const r = Math.min(shapeOf("slot").radius, Math.max(3, side / 9));
  const k = keyId(u0.key ?? u0.name ?? "");
  const max = u0.healthMax ?? 0;
  const hp = max > 0 ? clamp01((u0.health ?? 0) / max) : 1;
  const level = hp < 0.25 ? "critical" : hp < 0.5 ? "low" : null;
  const barH = Math.max(3, Math.round(side / 14));
  const inset = Math.max(2, Math.round(side / 18));
  open(k, { width: px(side), height: px(side), interactive: true, part: sel("SelectionPanel", "tile", focused ? "focused" : null), partClass: "slot" }, {
    shape: { kind: "rect", radii: [r, r, r, r] },
    fills: [solid(lit(BLACK, 0.9))],
    strokes: [stroke(1, focused ? token("highlight", 0.6) : token("lineStrong", 0.4), "outside")],
    // Focused, it lifts: a soft highlight glow round it (the selected-tile rule's lift).
    effects: focused ? [glow(u(7), token("highlight", 0.4))] : [],
    clip: "shape",
  });
  artTile("Portrait", u0.image, u0.icon ?? "", side, hueOf(u0.icon || u0.name || k), absolute({ start: 0, top: 0 }));
  // A dark foot under the bars, so they read over any art.
  const bars = (max > 0 ? 1 : 0) + ((u0.shieldMax ?? 0) > 0 ? 1 : 0) + ((u0.energyMax ?? 0) > 0 ? 1 : 0);
  if (bars > 0) {
    const footH = inset * 2 + bars * barH + (bars - 1);
    leaf("Foot", { width: pct(1), height: px(footH + inset * 2) }, { fills: [vgrad([[0, lit(BLACK, 0)], [0.45, lit(BLACK, 0.7)], [1, lit(BLACK, 0.85)]])] }, absolute({ start: 0, bottom: 0 }));
    let y = inset;
    const bar = (bid: string, f: number, ink: Rgba, part: string) => {
      box(bid, { width: px(side - inset * 2), height: px(barH), part }, { shape: { kind: "rect", radii: [1, 1, 1, 1] }, fills: [solid(lit(BLACK, 0.75))] }, () => {
        if (f > 0) leaf("Fill", { width: pct(f), height: pct(1) }, fillDraw(ink, 1));
      }, absolute({ start: inset, bottom: y }));
      y += barH + 1;
    };
    if ((u0.energyMax ?? 0) > 0) bar("Energy", clamp01((u0.energy ?? 0) / (u0.energyMax ?? 1)), ENERGY, sel("SelectionPanel", "energy"));
    if (max > 0) bar("Health", hp, healthInk(hp), sel("SelectionPanel", "health", level));
    if ((u0.shieldMax ?? 0) > 0) bar("Shield", clamp01((u0.shield ?? 0) / (u0.shieldMax ?? 1)), SHIELD, sel("SelectionPanel", "shield"));
  }
  const p = u0.progress ?? null;
  if (p !== null) {
    leaf("Progress", { width: pct(clamp01(p)), height: px(barH), part: sel("SelectionPanel", "progress") }, fillDraw(role("gold"), 0), absolute({ start: 0, top: 0 }));
  }
  // The selected-tile rule (docs/ui.md §15.5): the highlight ring inside the edge at a clearly visible width (3 px at
  // 1080p), the art lit from its rim inward, and the glow round the tile above.
  if (focused) leaf("Focus", { width: pct(1), height: pct(1), part: sel("SelectionPanel", "focus") }, { shape: { kind: "rect", radii: [r, r, r, r] }, strokes: [stroke(Math.max(2, u(2)), token("highlight"), "inside")], effects: [innerGlow(u(10), token("highlight", 0.3))] }, absolute({ start: 0, top: 0 }));
  else leaf("Rim", { width: pct(1), height: pct(1) }, { shape: { kind: "rect", radii: [r, r, r, r] }, strokes: [{ width: 1, align: "inside", paint: vgrad([[0, lit(WHITE, 0.28)], [0.5, lit(WHITE, 0.05)], [1, lit(BLACK, 0.6)]]) }] }, absolute({ start: 0, top: 0 }));
  const res = close({ a11y: a11y("button", u0.name ?? "", { value: max > 0 ? `${Math.round(u0.health ?? 0)} of ${Math.round(max)}` : "", selected: focused }) });
  kit.tooltip(u0.name ?? "");
  return res;
}

/** The grid of tiles; a selection larger than the grid pages, its last tile counting what is left. */
function tiles(desc: Api.SelectionPanelDesc, side: number, cols: number, rows: number, focus: string, onClick: (k: string) => void): void {
  const units = desc.units ?? [];
  const per = cols * rows;
  const pages = Math.max(1, Math.ceil(units.length / per));
  const page = Math.min(desc.page ?? 0, pages - 1);
  const pageUnits = units.slice(page * per, page * per + per);
  const more = units.length - (page * per + per);
  const shown = more > 0 ? pageUnits.slice(0, per - 1) : pageUnits;
  const gap = Math.max(2, u(4));
  box("Tiles", { direction: "row", wrap: true, gap, width: px(cols * side + (cols - 1) * gap), height: px(rows * side + (rows - 1) * gap) }, undefined, () => {
    for (const un of shown) {
      if (tile(un, side, (un.key ?? "") === focus).clicked) onClick(un.key ?? "");
    }
    if (more > 0) {
      const r = Math.min(shapeOf("slot").radius, Math.max(3, side / 9));
      box("More", { width: px(side), height: px(side), align: "center", cross: "center", part: sel("SelectionPanel", "more") }, well(r), () => {
        text("Count", `+${more + 1}`, { step: tsFit(13, side * 0.4, true), font: "display", color: token("inkMuted"), nowrap: true });
      }, { a11y: a11y("text", `${more + 1} more`) });
    }
  }, { a11y: a11y("list", "Selection") });
}

/** The focused unit's card: the portrait at full height, then name, type, health numeral over a bar, stats, tags. */
function unitCard(desc: Api.SelectionPanelDesc, un: Api.SelectedUnit, h: number, w: number, withStats = true): void {
  const tk = frame().t;
  const nameStep = ts(16, true);
  const kindStep = ts(11);
  const hpStep = ts(13, true);
  const statStep = ts(11);
  const tagStep = ts(10);
  const max = un.healthMax ?? 0;
  const hp = max > 0 ? clamp01((un.health ?? 0) / max) : 1;
  const k = keyId(un.key ?? un.name ?? "");
  const port = Math.min(h, Math.round(w * 0.4));
  const r = Math.min(shapeOf("slot").radius, Math.max(3, port / 12));
  kit.row("Detail", { gap: tk.md, cross: "start", width: px(w), height: px(h), part: sel("SelectionPanel", "detail") }, () => {
    box("Portrait", { width: px(port), height: px(h), part: sel("SelectionPanel", "portrait"), partClass: "slot" }, {
      shape: { kind: "rect", radii: [r, r, r, r] },
      fills: [solid(lit(BLACK, 0.9))],
      strokes: [stroke(1, token("lineStrong", 0.4), "outside")],
      clip: "shape",
    }, () => {
      artTile("Art", un.image, un.icon ?? "", Math.max(port, h), hueOf(un.icon || un.name || k), absolute({ start: (port - Math.max(port, h)) / 2, top: 0 }));
      leaf("Rim", { width: pct(1), height: pct(1) }, { shape: { kind: "rect", radii: [r, r, r, r] }, strokes: [{ width: 1, align: "inside", paint: vgrad([[0, lit(WHITE, 0.28)], [0.5, lit(WHITE, 0.05)], [1, lit(BLACK, 0.6)]]) }] }, absolute({ start: 0, top: 0 }));
    });
    const infoW = w - port - tk.md;
    // As many stat rows as the card's height holds at this text size; the tags last, at the foot.
    const line = (st: Api.TypeStepName) => Math.ceil(stepSize(st).line);
    const barH = Math.max(4, u(5));
    const fixedH = line(nameStep) + line(kindStep) + u(4) + line(hpStep) + barH + u(6);
    // The tags only where they fit under the health (a building's card, shortened by its queue, may drop them).
    const tagH = withStats && (desc.tags ?? []).length > 0 && fixedH + line(tagStep) + u(4) <= h ? line(tagStep) + u(4) : 0;
    const statLine = line(statStep) + u(2);
    const nStats = !withStats ? 0 : Math.max(0, Math.min((desc.stats ?? []).length, Math.floor((h - fixedH - tagH) / statLine)));
    kit.column("Info", { gap: 0, width: px(infoW), height: px(h) }, () => {
      text("Name", un.name ?? "", { step: nameStep, font: "display", color: token("ink"), nowrap: true, ellipsis: true, width: pct(1) });
      if (un.kind) text("Kind", un.kind, { step: kindStep, color: token("inkMuted"), nowrap: true, ellipsis: true, width: pct(1) });
      if (max > 0) {
        kit.row("Health", { gap: u(3), cross: "end", padding: { left: 0, right: 0, top: u(4), bottom: u(2) } }, () => {
          text("Value", `${Math.round(un.health ?? 0)}`, { step: hpStep, font: "mono", color: lit(mix(healthInk(hp), WHITE, 0.25)), nowrap: true });
          text("Max", `/ ${Math.round(max)}`, { step: ts(11), font: "mono", color: token("inkMuted"), nowrap: true });
        });
        box("Bar", { width: pct(1), height: px(barH), part: sel("SelectionPanel", "health") }, { shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [solid(lit(BLACK, 0.7))] }, () => {
          if (hp > 0) leaf("Fill", { width: pct(hp), height: pct(1) }, fillDraw(healthInk(hp), 2));
        });
        if ((un.shieldMax ?? 0) > 0) {
          box("Shield", { width: pct(1), height: px(Math.max(2, u(2))), part: sel("SelectionPanel", "shield") }, { fills: [solid(lit(BLACK, 0.6))] }, () => {
            leaf("Fill", { width: pct(clamp01((un.shield ?? 0) / (un.shieldMax ?? 1))), height: pct(1) }, { fills: [solid(lit(SHIELD))] });
          });
        }
      }
      if (nStats > 0) {
        kit.column("Stats", { gap: u(2), width: pct(1), padding: { left: 0, right: 0, top: u(6), bottom: 0 } }, () => {
          for (const st of (desc.stats ?? []).slice(0, nStats)) {
            kit.row(keyId(st.key ?? st.label ?? ""), { gap: u(6), cross: "center", width: pct(1), part: sel("SelectionPanel", "stat") }, () => {
              if (st.icon) icon("Icon", st.icon, Math.round(stepSize(statStep).size * 1.05), token("inkMuted"));
              text("Label", st.label ?? "", { step: statStep, color: token("inkMuted"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
              text("Value", st.value ?? "", { step: statStep, font: "mono", color: token("ink"), nowrap: true });
            });
          }
        });
      }
      if (tagH > 0) {
        leaf("Spacer", { width: px(1), height: { kind: "grow" } });
        text("Tags", (desc.tags ?? []).join("  ·  "), { step: tagStep, color: token("inkMuted"), nowrap: true, ellipsis: true, width: pct(1) });
      }
    });
  });
}

/** A single selection's stats in the grid's zone: label and value pairs in as many columns as fit, the tags at the foot. */
function statsZone(desc: Api.SelectionPanelDesc, h: number, w: number): void {
  const step = ts(12);
  const tagStep = ts(11);
  const stats = desc.stats ?? [];
  const tags = desc.tags ?? [];
  const line = Math.ceil(stepSize(step).line) + u(6);
  const colGap = u(24);
  const tagH = tags.length > 0 ? Math.ceil(stepSize(tagStep).line) + u(6) : 0;
  const perCol = Math.max(1, Math.floor((h - tagH) / line));
  const cols = Math.max(1, Math.min(Math.ceil(stats.length / perCol), Math.floor((w + colGap) / (ut(150) + colGap))));
  const colW = Math.floor((w - (cols - 1) * colGap) / cols);
  const shown = stats.slice(0, cols * perCol);
  kit.column("Stats", { gap: 0, width: px(w), height: px(h), part: sel("SelectionPanel", "stats") }, () => {
    kit.row("Columns", { gap: colGap, cross: "start", width: pct(1) }, () => {
      for (let c = 0; c < cols; c++) {
        kit.column(kit.idi("Col", c), { gap: u(6), width: px(colW) }, () => {
          for (const st of shown.slice(c * perCol, c * perCol + perCol)) {
            kit.row(keyId(st.key ?? st.label ?? ""), { gap: u(8), cross: "center", width: pct(1), part: sel("SelectionPanel", "stat") }, () => {
              if (st.icon) icon("Icon", st.icon, Math.round(stepSize(step).size * 1.05), token("inkMuted"));
              text("Label", st.label ?? "", { step, color: token("inkMuted"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
              text("Value", st.value ?? "", { step, font: "mono", color: token("ink"), nowrap: true });
            });
          }
        });
      }
    });
    if (tagH > 0) {
      leaf("Spacer", { width: px(1), height: { kind: "grow" } });
      text("Tags", tags.join("  ·  "), { step: tagStep, color: token("inkMuted"), nowrap: true, ellipsis: true, width: pct(1) });
    }
  });
}

/** The unit card as one line, for a screen with no room for the card: name, type, health. */
function unitLine(un: Api.SelectedUnit, w: number): void {
  const max = un.healthMax ?? 0;
  const hp = max > 0 ? clamp01((un.health ?? 0) / max) : 1;
  kit.row("Detail", { gap: u(6), cross: "center", width: px(w), part: sel("SelectionPanel", "detail") }, () => {
    text("Name", un.name ?? "", { step: ts(13, true), font: "display", color: token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
    if (max > 0) {
      kit.row("Health", { gap: u(2), cross: "end" }, () => {
        text("Value", `${Math.round(un.health ?? 0)}`, { step: ts(12), font: "mono", color: lit(mix(healthInk(hp), WHITE, 0.25)), nowrap: true });
        text("Max", `/ ${Math.round(max)}`, { step: ts(10), font: "mono", color: token("inkMuted"), nowrap: true });
      });
    }
  });
}

/**
 * A building's production beside its card (StarCraft II's single selection): the item being built in a slot the size
 * of a command cell, its sweep over the art and its time under it, its name over a progress bar beside it, then the
 * rest of the queue as smaller slots under them. The building's card stays the zone's one large image.
 */
function queueZone(desc: Api.ProductionQueueDesc, h: number, w: number, small: number, active: number): Api.ProductionQueueResult {
  const tk = frame().t;
  const items = desc.items ?? [];
  const first = items[0];
  const gap = u(6);
  const paused = !!desc.paused;
  let cancelled: string | null = null;
  kit.column("Production", { gap, width: px(w), height: px(h), part: sel("ProductionQueue", "zone") }, () => {
    kit.row("Current", { gap: tk.md, cross: "start", width: pct(1) }, () => {
      kit.column("Active", { gap: u(2), cross: "center" }, () => {
        const r = queueRow("Queue", desc, active, 0, 1, false);
        if (r.cancelled) cancelled = r.cancelled;
        if (first) text("Time", queueTime(desc, first), { step: ts(12), font: "mono", color: paused ? lit(role("warn")) : token("ink"), nowrap: true, outline: 1 });
      });
      if (!first) return;
      const dur = Math.max(1e-3, first.duration ?? 0);
      const done = (first.duration ?? 0) > 0 ? clamp01(1 - (first.remaining ?? 0) / dur) : 0;
      const barH = Math.max(4, u(5));
      kit.column("Info", { gap: u(4), width: { kind: "grow" }, padding: { left: 0, right: 0, top: u(4), bottom: 0 } }, () => {
        text("Name", first.name ?? "", { step: ts(13, true), font: "display", color: token("ink"), nowrap: true, ellipsis: true, width: pct(1) });
        box("Bar", { width: pct(1), height: px(barH), part: sel("ProductionQueue", "bar", paused ? "paused" : null) }, { shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [solid(lit(BLACK, 0.7))] }, () => {
          if (done > 0) leaf("Fill", { width: pct(done), height: pct(1) }, fillDraw(role(paused ? "warn" : "gold"), 2));
        });
      });
    });
    const r = queueRow("Next", desc, small, 1);
    if (r.cancelled) cancelled = r.cancelled;
  });
  return { cancelled };
}

interface SelLayout { side: number; cols: number; rows: number; card: number; compact: boolean; queueSide: number }

/** The selection's zones in place (tiles, a rule, the card with its queue above), with no panel and no tabs. */
function selectionBody(desc: Api.SelectionPanelDesc, h: number, lay: SelLayout, res: Mut<Api.SelectionPanelResult>): void {
  const tk = frame().t;
  const units = desc.units ?? [];
  const focus = desc.focus || units[0]?.key || "";
  const focused = units.find((x) => (x.key ?? "") === focus) ?? units[0];
  const queue = desc.queue && (desc.queue.items ?? []).length > 0 ? desc.queue : null;
  // One unit or building: no grid of one, the card takes the selection zone's whole width (StarCraft II's single selection).
  const single = units.length === 1;
  const gap = Math.max(2, u(4));
  const gridW = lay.cols * lay.side + (lay.cols - 1) * gap;
  if (single && focused && !lay.compact) {
    // The card at its own width; a building's production takes the grid's zone, a unit's card spans both.
    kit.row("Body", { gap: tk.md, cross: "start", width: px(gridW + tk.md * 2 + 1 + lay.card) }, () => {
      if (queue) {
        kit.column("Card", { width: px(lay.card) }, () => unitCard(desc, focused, h, lay.card));
        rule("QueueRule", h);
        // The item being built in a slot the size of a command cell (the console's three rows in its height), the
        // waiting ones in slots a step smaller.
        const active = Math.round((h - 2 * gap) / 3);
        const q = queueZone(queue, h, gridW, Math.max(u(24), Math.round(active * 0.72)), active);
        if (q.cancelled) res.cancelled = q.cancelled;
      } else {
        // A unit or a building with nothing queued: the card keeps its width, and its stats and tags move into the
        // grid's zone in columns, so neither a stretched card nor an empty zone stands beside it.
        kit.column("Card", { width: px(lay.card) }, () => unitCard(desc, focused, h, lay.card, false));
        rule("StatsRule", h);
        statsZone(desc, h, gridW);
      }
    });
    return;
  }
  if (single && focused) {
    // Compact: the one tile, its line and its queue beside it, so no grid of empty rows stands under a single tile.
    kit.row("Body", { gap: tk.md, cross: "start", width: px(gridW) }, () => {
      kit.column("Grid", {}, () => tiles(desc, lay.side, 1, 1, focus, (k) => { res.clicked = k; }));
      kit.column("Card", { gap: u(6), width: px(Math.max(0, gridW - lay.side - tk.md)) }, () => {
        unitLine(focused, Math.max(0, gridW - lay.side - tk.md));
        if (queue) {
          const q = queueRow("Queue", queue, lay.queueSide);
          if (q.cancelled) res.cancelled = q.cancelled;
        }
      });
    });
    return;
  }
  // A grid no taller than its units need (a phone's two rows hold a squad of three in one).
  const rows = lay.compact ? Math.max(1, Math.min(lay.rows, Math.ceil(units.length / lay.cols))) : lay.rows;
  kit.row("Body", { gap: tk.md, cross: "start" }, () => {
    kit.column("Grid", { gap: u(6) }, () => {
      tiles(desc, lay.side, lay.cols, rows, focus, (k) => { res.clicked = k; });
      if (lay.compact && focused) unitLine(focused, lay.cols * lay.side + (lay.cols - 1) * Math.max(2, u(4)));
      if (lay.compact && queue) {
        const q = queueRow("Queue", queue, lay.queueSide);
        if (q.cancelled) res.cancelled = q.cancelled;
      }
    });
    if (!lay.compact && focused) {
      rule("Rule", h);
      kit.column("Card", { gap: u(6), width: px(lay.card) }, () => {
        let ch = h;
        if (queue) {
          const q = queueRow("Queue", queue, lay.queueSide);
          if (q.cancelled) res.cancelled = q.cancelled;
          ch -= lay.queueSide + u(6);
        }
        unitCard(desc, focused, ch, lay.card);
      });
    }
  }, );
}

function selLayout(desc: Api.SelectionPanelDesc, h: number, room: number, compact: boolean): SelLayout {
  const tk = frame().t;
  const rows = Math.max(1, desc.rows || 2);
  const gap = Math.max(2, u(4));
  const side = Math.floor((h - gap * (rows - 1)) / rows);
  const card = compact ? 0 : Math.max(ut(240), Math.round(h * 1.8));
  const want = desc.columns || (bp().phone ? 4 : 6);
  // room is the selection zone's whole width: the card's and its rule's share comes off it first.
  const left = room - (compact ? 0 : card + tk.md * 2 + 1);
  const cols = Math.max(2, Math.min(want, Math.floor((left + gap) / (side + gap))));
  return { side, cols, rows, card, compact, queueSide: Math.max(u(24), Math.round(h * 0.26)) };
}

function selWidth(lay: SelLayout): number {
  const gap = Math.max(2, u(4));
  return lay.cols * lay.side + (lay.cols - 1) * gap + (lay.compact ? 0 : lay.card + frame().t.md * 2 + 1);
}

export function selectionPanel(id: string, desc: Api.SelectionPanelDesc = {}): Api.SelectionPanelResult {
  const tk = frame().t;
  const res: Mut<Api.SelectionPanelResult> = { clicked: null, group: null, cancelled: null };
  const anchor = anchorOf(desc, "bottom");
  const phone = bp().phone;
  const h = u(phone ? 96 : 140);
  const room = hudRoom() - tk.md * 2;
  const lay = selLayout(desc, h, room, phone || selWidth(selLayout(desc, h, room, false)) > room);
  const n = (desc.units ?? []).length;
  hudPlace(id, anchor, (inAnchor) => {
    kit.column(id, { gap: 0, cross: "start" }, () => {
      tabs(desc, tabHeight(h), (k) => { res.group = k; }, inAnchor ? `${kit.openPath()}/Panel` : "");
      if (inAnchor) {
        box("Panel", { padding: { left: tk.md, right: tk.md, top: tk.md, bottom: tk.md }, part: sel("SelectionPanel", "frame"), partClass: "hud" }, panelDraw(), () => selectionBody(desc, h, lay, res));
      } else selectionBody(desc, h, lay, res);
    }, );
    kit.semantics(a11y("group", "Selection"));
    kit.props(groups({ selection: n === 0 ? "empty" : n === 1 ? "single" : "many" }, { selection: ["empty", "single", "many"] }));
  });
  return res;
}

// ── command card ──────────────────────────────────────────────────────

/** The command grid's rows: three (StarCraft's 5 × 3) so the card keeps its height as the selection changes, more if the commands need them; `trim` drops trailing empty rows (a phone, where the card stacks). */
function gridRows(desc: Api.CommandCardDesc, trim = false): number {
  const cmds = desc.commands ?? [];
  const cols = Math.max(1, desc.columns || 5);
  if (!trim) return Math.max(3, Math.ceil(cmds.length / cols));
  let last = -1;
  cmds.forEach((c, i) => { if (c.key) last = i; });
  return Math.max(1, Math.ceil((last + 1) / cols));
}

function cardGrid(desc: Api.CommandCardDesc, cell: number, trim = false): Api.AbilityBarResult {
  const cmds = desc.commands ?? [];
  const cols = Math.max(1, desc.columns || 5);
  const rows = gridRows(desc, trim);
  const gap = Math.max(2, u(4));
  const r = Math.min(shapeOf("slot").radius, Math.max(3, cell / 10));
  let clicked: string | null = null;
  const pressed: string[] = [];
  kit.column("Grid", { gap }, () => {
    for (let y = 0; y < rows; y++) {
      kit.row(kit.idi("Row", y), { gap }, () => {
        for (let x = 0; x < cols; x++) {
          const c = cmds[y * cols + x];
          if (!c || !c.key) {
            leaf(kit.idi("Empty", y * cols + x), { width: px(cell), height: px(cell), part: sel("CommandCard", "empty") }, well(r));
            continue;
          }
          const resp = slot(keyId(c.key), c, cell, true);
          if (resp.clicked) clicked = c.key;
          if (resp.pressed) pressed.push(c.key);
        }
      });
    }
  });
  return { clicked, pressed };
}

function cellSize(desc: Api.CommandCardDesc): number {
  return u(desc.size || (bp().phone ? 40 : 50));
}

export function commandCard(id: string, desc: Api.CommandCardDesc = {}): Api.AbilityBarResult {
  const tk = frame().t;
  let res: Api.AbilityBarResult = { clicked: null, pressed: [] };
  const cell = cellSize(desc);
  hudPlace(id, anchorOf(desc, "bottomRight"), (inAnchor) => {
    box(id, { padding: inAnchor ? { left: tk.sm, right: tk.sm, top: tk.sm, bottom: tk.sm } : { left: 0, right: 0, top: 0, bottom: 0 }, part: sel("CommandCard", "frame"), partClass: "hud" }, inAnchor ? panelDraw() : undefined, () => {
      res = cardGrid(desc, cell);
    }, { a11y: a11y("group", "Commands") });
  });
  return res;
}

// ── the console ───────────────────────────────────────────────────────

export function rtsConsole(id: string, desc: Api.RtsConsoleDesc = {}): Api.RtsConsoleResult {
  const tk = frame().t;
  const out: Mut<Api.RtsConsoleResult> = { mapClicked: false, clicked: null, group: null, cancelled: null, command: null, pressed: [] };
  const sd: Api.SelectionPanelDesc = desc.selection ?? {};
  const cd: Api.CommandCardDesc = desc.commands ?? {};
  const cell = cellSize(cd);
  const gap = Math.max(2, u(4));
  const cols = Math.max(1, cd.columns || 5);
  const rows = gridRows(cd);
  const cmdW = cols * cell + (cols - 1) * gap;
  const h = rows * cell + (rows - 1) * gap;
  const pad = tk.md;
  const room = hudRoom();
  const zone = pad * 2 + 1;
  // What fits, in order: the four zones; then the minimap out to the game's corner; then the card as a line; then
  // the command card over the selection.
  // What fits, in order: the four zones with the selection grid narrowed to what is left (at least four columns);
  // then the minimap out to the game's corner; then the card as a line; then the command card over the selection.
  const selRoom = (m: boolean) => room - pad * 2 - (m ? h + zone : 0) - zone - cmdW;
  let withMap = !!desc.map;
  let lay = selLayout(sd, h, selRoom(withMap), false);
  const wide = (l: SelLayout, m: boolean) => pad * 2 + (m ? h + zone : 0) + selWidth(l) + zone + cmdW;
  if (withMap && (lay.cols < 4 || wide(lay, true) > room)) {
    withMap = false;
    lay = selLayout(sd, h, selRoom(false), false);
  }
  if (lay.cols < 4 || wide(lay, withMap) > room) lay = selLayout(sd, h, selRoom(withMap), true);
  const stacked = wide(lay, withMap) > room;
  if (stacked) lay = selLayout(sd, Math.round(h * 0.82), room - pad * 2, true);
  if (desc.map && !withMap) {
    // Out of the console: beside it at the bottom start where the band leaves it the width (a phone held sideways,
    // as an RTS keeps its map in that corner), else in the top end corner, no larger than the height left above the
    // console and the corner's stack (last tick's sizes).
    const vp = frame().viewport;
    const console = kit.bounds(`${id}Hud`);
    const consoleH = console?.height ?? 0;
    const k = u(100) / 100;
    const want = desc.map.size || (bp().phone ? 144 : 200);
    const chrome = tk.sm * 2 + u(28);
    const besidePx = Math.min((room - (console?.width ?? room)) / 2 - tk.gutter, consoleH) - chrome;
    // What the corner holds before the map this tick (its reach so far, plus the gap the map would start after):
    // the stack's total of last tick counts the map itself, and sizing the map by it would make it flicker.
    const placed = kit.memory("~hudStack/topRight/tick", -1) === kit.tick();
    const above = placed ? kit.memory("~hudStack/topRight/reach", 0) + tk.sm : 0;
    const topPx = vp.height - vp.safeArea.top - vp.safeArea.bottom - consoleH - above - tk.gutter * 2 - chrome;
    const beside = Math.min(want, Math.floor(besidePx / k));
    const top = Math.min(want, Math.floor(topPx / k));
    const [size, anchor] = beside >= 64 && beside >= top ? [beside, "bottomLeft" as const] : [top, "topRight" as const];
    if (size >= 64) {
      const m = minimapView(`${id}Map`, { ...desc.map, size, anchor });
      out.mapClicked = m.clicked;
    }
  }
  const tabH = tabHeight(h);
  const sres: Mut<Api.SelectionPanelResult> = { clicked: null, group: null, cancelled: null };
  hudPlace(id, anchorOf(desc, "bottom"), () => {
    kit.column(id, { gap: 0, cross: "start" }, () => {
      // The tabs stand on the panel's top edge over the selection zone.
      const panelPath = `${kit.openPath()}/Panel`;
      kit.row("TabRow", { padding: { left: pad + (withMap && !stacked ? h + zone : 0), right: 0, top: 0, bottom: 0 } }, () => tabs(sd, tabH, (k) => { sres.group = k; }, panelPath));
      kit.props({ z: 1 });
      box("Panel", { direction: stacked ? "column" : "row", gap: stacked ? tk.sm : pad, cross: stacked ? "start" : "center", padding: { left: pad, right: pad, top: pad, bottom: pad }, part: sel("RtsConsole", "frame"), partClass: "hud" }, panelDraw(), () => {
        if (withMap && desc.map) {
          kit.column("Map", { width: px(h), height: px(h), cross: "center" }, () => {
            // An RTS map is square: it fills the zone (less its caption's line), its north mark inside its top edge.
            const m = minimapView("View", { ...desc.map!, shape: "square", anchor: null, size: Math.max(40, (h - (desc.map!.zone ? Math.ceil(stepSize(ts(11)).line) + u(4) : 0)) / Math.max(0.5, u(100) / 100)) }, true);
            out.mapClicked = m.clicked;
          });
          rule("MapRule", h);
        }
        if (stacked) {
          box("Commands", {}, undefined, () => {
            const r = cardGrid(cd, cell, true);
            out.command = r.clicked;
            out.pressed = r.pressed;
          }, { a11y: a11y("group", "Commands") });
          leaf("StackRule", { width: pct(1), height: px(1), part: sel("RtsConsole", "rule") }, { fills: [solid(token("line", 0.9))] });
          kit.column("Selection", {}, () => selectionBody(sd, Math.round(h * 0.82), lay, sres));
        } else {
          kit.column("Selection", {}, () => selectionBody(sd, h, lay, sres));
          rule("CmdRule", h);
          box("Commands", {}, undefined, () => {
            const r = cardGrid(cd, cell);
            out.command = r.clicked;
            out.pressed = r.pressed;
          }, { a11y: a11y("group", "Commands") });
        }
      });
    }, );
    kit.semantics(a11y("group", "Console"));
  });
  out.clicked = sres.clicked;
  out.group = sres.group;
  out.cancelled = sres.cancelled;
  return out;
}

// ── drag selection ────────────────────────────────────────────────────

export function dragSelect(id: string, desc: Api.DragSelectDesc = {}): Api.DragSelectResult {
  const vp = frame().viewport;
  const band = vp.hud ?? { x: 0, y: 0, width: vp.width, height: vp.height };
  const res: Mut<Api.DragSelectResult> = { dragging: false, released: false, x: 0, y: 0, width: 0, height: 0 };
  let a: { x: number; y: number } | null = null;
  let b: { x: number; y: number } | null = null;
  if (desc.from && desc.to) {
    a = { x: desc.from.x ?? 0, y: desc.from.y ?? 0 };
    b = { x: desc.to.x ?? 0, y: desc.to.y ?? 0 };
    res.dragging = true;
  } else if (desc.enabled !== false) {
    const p = kit.framePointer();
    const k = `~drag/${id}`;
    const was = kit.memory(`${k}/down`, 0) === 1;
    const armed = kit.memory(`${k}/armed`, 0) === 1;
    const sx = kit.memory(`${k}/x`, 0);
    const sy = kit.memory(`${k}/y`, 0);
    const threshold = desc.threshold ?? 6;
    if (p.down && !was) {
      // A press on a HUD root named in exclude is that element's.
      const onHud = (desc.exclude ?? []).some((path) => {
        const r = kit.bounds(path);
        return !!r && p.x >= r.x && p.x <= r.x + r.width && p.y >= r.y && p.y <= r.y + r.height;
      });
      kit.remember(`${k}/armed`, onHud || !p.inside ? 0 : 1);
      kit.remember(`${k}/x`, p.x);
      kit.remember(`${k}/y`, p.y);
      kit.remember(`${k}/live`, 0);
    } else if (armed) {
      const live = kit.memory(`${k}/live`, 0) === 1 || Math.hypot(p.x - sx, p.y - sy) >= threshold;
      kit.remember(`${k}/live`, live ? 1 : 0);
      if (live) {
        a = { x: sx, y: sy };
        b = { x: p.x, y: p.y };
        if (p.down) res.dragging = true;
        else {
          res.released = true;
          kit.remember(`${k}/armed`, 0);
          kit.remember(`${k}/live`, 0);
        }
      }
      if (!p.down && !live) kit.remember(`${k}/armed`, 0);
    }
    kit.remember(`${k}/down`, p.down ? 1 : 0);
  }
  if (!a || !b) return res;
  res.x = Math.min(a.x, b.x);
  res.y = Math.min(a.y, b.y);
  res.width = Math.abs(a.x - b.x);
  res.height = Math.abs(a.y - b.y);
  if (!res.dragging) return res;
  const ink: Rgba = desc.color ? ((desc.color as { color?: Rgba }).color ?? role("info")) : role("info");
  const w = Math.max(1, res.width);
  const hh = Math.max(1, res.height);
  const tick = Math.min(u(10), w / 3, hh / 3);
  kit.anchor(`${id}Hud`, { anchor: "topLeft", margin: 0, offsetX: res.x - band.x, offsetY: res.y - band.y, direction: "column" }, () => {
    box(id, { width: px(w), height: px(hh), part: sel("DragSelect", "box") }, {
      fills: [solid(lit(ink, 0.12))],
      strokes: [stroke(1, lit(ink, 0.85))],
    }, () => {
      const t = Math.max(2, u(2));
      const corners: Array<[number, number, number, number]> = [[0, 0, 1, 1], [w, 0, -1, 1], [0, hh, 1, -1], [w, hh, -1, -1]];
      corners.forEach(([cx, cy, dx, dy], i) => {
        leaf(kit.idi("Corner", i), { width: px(w), height: px(hh), part: sel("DragSelect", "corner") }, {
          shape: { kind: "path", points: [{ x: cx, y: cy + dy * tick }, { x: cx, y: cy }, { x: cx + dx * tick, y: cy }], width: t, cap: "butt", join: "miter" },
          fills: [solid(lit(mix(ink, WHITE, 0.4)))],
        }, absolute({ start: 0, top: 0 }));
      });
    });
  });
  return res;
}
