// The world map (docs/ui.md §11 genre patterns, Appendix B: world map with
// fog, markers and fast travel). Read against The Witcher 3's (the painted
// map filling the screen, the legend at the side turning kinds off, a pin's
// detail with its action), Breath of the Wild's (regions named across the
// map while zoomed out, the places' names once zoomed in) and Genshin
// Impact's (the fog over what is not found, the waypoint's teleport). The
// map's zoom and centre are og.ui.zoomView's (UI state); what is found,
// shown and tracked is the game's.

import type * as Api from "@origingame/engine";
import {
  firstRoom,
  artSlot, btn, clamp, eyebrow, flags, frame, glow, grow, keyHash, kit, lastInteraction, line, over, paint, px, rememberRoom, role, safeHeight, safeWidth, shadow, spring,
  st, still, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut, type Rgba, type SymbolKind,
} from "./core.ts";
import { nameText } from "../feedback/core.ts";

/** A kind's mark and ink: the main quest's crown in gold, a fast-travel point's bolt, a shop's coin, a boss's sword in danger. */
function kindMark(kind: string): { sym: SymbolKind; ink: Rgba } {
  switch (kind) {
    case "quest":
      return { sym: "crown", ink: role("gold") };
    case "objective":
      return { sym: "star", ink: role("gold") };
    case "fastTravel":
      return { sym: "bolt", ink: role("info") };
    case "shop":
      return { sym: "coin", ink: role("gold") };
    case "boss":
      return { sym: "sword", ink: role("danger") };
    case "custom":
      return { sym: "pin", ink: role("success") };
    default:
      return { sym: "eye", ink: role("ink") };
  }
}

/** A word from the game's map of words, else the key. */
function word(words: unknown, key: string): string {
  const w = words && typeof words === "object" ? (words as Record<string, unknown>)[key] : undefined;
  return typeof w === "string" && w ? w : key;
}

/**
 * A world map (The Witcher 3, Breath of the Wild, Genshin Impact): see
 * `fn worldMap` in crates/og-api/api/ui.ogt.
 */
export function worldMap(id: string, desc: Api.WorldMapDesc): Api.WorldMapResult {
  const out: Mut<Api.WorldMapResult> = { selected: null, travel: null, track: null, toggled: null, zoom: 1, centerX: 0.5, centerY: 0.5 };
  const { t, phone } = frame();
  const tick = kit.tick();
  const W = kit.memory(`~w.w.${id}`, 0) || firstRoom().w;
  const H = kit.memory(`~w.h.${id}`, 0) || firstRoom().h;
  const aspect = (desc.aspect ?? 0) > 0 ? desc.aspect! : 16 / 9;
  const hidden = new Set(desc.hidden ?? []);
  const markers = desc.markers ?? [];
  const regions = desc.regions ?? [];
  const sel = markers.find((m) => m.key === desc.selected) ?? null;
  // The map covers the frame (as wide as the frame or as tall, whichever covers it) where that cuts little of it; where it
  // would cut much (a phone in portrait) it sits whole across the frame's width, zoomed into from there.
  const cover = Math.max(W, H * aspect);
  const mw = cover / Math.min(W, H * aspect) <= 1.2 ? cover : Math.min(W, H * aspect);
  const mh = mw / aspect;

  kit.open(id, { direction: "column", width: grow(), height: grow(), part: "WorldMap.frame", partClass: "panel" });
  kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow") }], clip: "shape" } as Api.DrawDesc);

  // The map, its fog, its names and its pins, laid over it by its zoom and centre.
  kit.open("Map", { width: px(W), height: px(H), part: "WorldMap.map", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect" }, clip: "shape" } as Api.DrawDesc);
  const z = kit.zoomView("View", { texture: desc.texture ?? null, width: px(mw), height: px(mh), minZoom: 1, maxZoom: (desc.maxZoom ?? 0) > 0 ? desc.maxZoom! : 4, tint: desc.texture ? null : token("well") } as Api.ZoomViewDesc) as Api.ZoomResult;
  kit.semantics({ role: "image", name: desc.title || "Map", value: `${Math.round(z.zoom * 100)} %` });
  // The view as large as the map, its middle on the frame's middle.
  const ox = (W - mw) / 2;
  const oy = (H - mh) / 2;
  over({ insets: { start: ox, top: oy } });
  out.zoom = z.zoom;
  out.centerX = z.centerX;
  out.centerY = z.centerY;
  const at = (x: number, y: number): { x: number; y: number } => ({ x: ox + mw * (0.5 + (x - z.centerX) * z.zoom), y: oy + mh * (0.5 + (y - z.centerY) * z.zoom) });
  const onScreen = (p: { x: number; y: number }, m = 0): boolean => p.x >= -m && p.y >= -m && p.x <= W + m && p.y <= H + m;
  // Names and labels by zoom: regions while zoomed out, places' labels from twice in.
  const regionA = clamp(1.9 - z.zoom * 0.7, 0, 1);
  const labelA = clamp((z.zoom - 1.6) / 0.6, 0, 1);

  // Fog: a cloud over each undiscovered region, darkest at its middle.
  for (const r of regions) {
    if (r.discovered) continue;
    const c = at(r.x ?? 0.5, r.y ?? 0.5);
    const rad = mw * z.zoom * ((r.radius ?? 0) > 0 ? r.radius! : 0.18);
    if (!onScreen(c, rad)) continue;
    kit.open(`Fog${r.key}`, { width: px(rad * 2), height: px(rad * 2), part: "WorldMap.fog", partClass: "badge" });
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "gradient", type: "radial", from: { x: 0.5, y: 0.5 }, to: { x: 0.5, y: 0.5 }, stops: [{ offset: 0, color: token("shadow", 0.88) }, { offset: 0.55, color: token("shadow", 0.74) }, { offset: 1, color: token("shadow", 0) }] }] });
    kit.close();
    over({ insets: { start: c.x - rad, top: c.y - rad }, inert: true, z: 1 });
  }
  // Region names across the map, fading as it zooms in. A name that would cross one already set (a small map, a phone's,
  // zoomed out) waits until the zoom parts them: discovered regions first, then the game's order. Each name's size is
  // kept from when it was last set, so a name held back stays held back rather than flickering.
  if (regionA > 0.02) {
    const placed: Array<{ x0: number; y0: number; x1: number; y1: number }> = [];
    // An undiscovered region is its fog alone: no name floats over it.
    for (const r of regions.filter((g) => g.discovered)) {
      const c = at(r.x ?? 0.5, r.y ?? 0.5);
      if (!onScreen(c, u(200))) continue;
      const nw = kit.memory(`~w.rw.${r.key}`, 0);
      const nh = kit.memory(`~w.rh.${r.key}`, 0);
      const box = { x0: c.x - nw / 2 - u(6), y0: c.y - u(14), x1: c.x + nw / 2 + u(6), y1: c.y - u(14) + nh };
      if (nw > 0 && placed.some((b) => box.x0 < b.x1 && b.x0 < box.x1 && box.y0 < b.y1 && b.y0 < box.y1)) continue;
      placed.push(box);
      kit.open(`Region${r.key}`, { direction: "column", cross: "center", width: px(u(320)), part: "WorldMap.region[discovered]", partClass: "badge" });
      // Names on the painting take the HUD's name treatment (an outline and a soft drop), so they read on any ground.
      text("Name", (r.name ?? "").toLocaleUpperCase(), { step: st(phone ? 15 : 22), font: "display", nowrap: true, color: "ink" });
      kit.setDraw({ text: nameText(u(phone ? 15 : 22)) });
      const nb = kit.bounds(kit.lastPath());
      kit.close();
      if (nb && nb.width > 0) {
        kit.remember(`~w.rw.${r.key}`, Math.round(nb.width));
        kit.remember(`~w.rh.${r.key}`, Math.round(nb.height));
      }
      over({ insets: { start: c.x - u(160), top: c.y - u(14) }, inert: true, z: 2, opacity: regionA });
    }
  }

  // Pins.
  kit.open("Pins", { width: px(W), height: px(H) });
  for (const m of markers) {
    if (hidden.has(m.kind ?? "")) continue;
    const p = at(m.x ?? 0, m.y ?? 0);
    if (!onScreen(p, u(40))) continue;
    const key = m.key ?? "";
    const found = m.discovered !== false;
    const mk = kindMark(m.kind ?? "");
    const isSel = !!sel && sel.key === key;
    const ix = lastInteraction(key);
    const size = u(phone ? 34 : 38) * (isSel ? 1.15 : 1);
    const pad = Math.max(0, Math.ceil((44 - size) / 2));
    const r = surface(key, {
      part: "WorldMap.pin",
      cls: "badge",
      flags: [m.kind ?? "poi", m.tracked ? "tracked" : "", found ? "" : "undiscovered", isSel ? "selected" : ""],
      fill: null,
      direction: "row",
      gap: u(6),
      cross: "center",
      // A whole touch target round the disc (a phone's pins are drawn smaller than a finger).
      padding: xy(pad, pad),
      radius: 999,
      lift: -u(3),
      semantics: { role: "button", name: [found ? m.name ?? key : desc.undiscoveredLabel ?? "???", word(desc.kindLabels, m.kind ?? ""), m.tracked ? "tracked" : ""].filter(Boolean).join(", "), selected: isSel },
      // Placed over the map with its states (a later over() would replace them: the pin lost its hover, press and lift).
      layer: {
        insets: { start: p.x - size / 2 - pad, top: p.y - size / 2 - pad },
        z: isSel ? 4 : 3,
        states: [{ name: "find", states: [vstate("discovered"), vstate("undiscovered")], transitions: [] }],
        state: [{ group: "find", state: found ? "discovered" : "undiscovered" }],
      },
    }, () => {
      // The pin: a disc of the panel with a rim in its kind's ink and its mark inside; tracked, a ring pulsing round it.
      kit.open("Disc", { width: px(size), height: px(size), align: "center", cross: "center" });
      const lit = isSel || ix.hovered || ix.focused;
      kit.setDraw({
        shape: { kind: "ellipse" },
        fills: [vgrad([[0, token("panelRaised", found ? 0.96 : 0.6)], [1, token("panel", found ? 0.96 : 0.6)]])],
        strokes: [line(u(lit ? 2.5 : 2), paint(found ? mk.ink : role("inkFaint"), found ? 1 : 0.6)), line(1, token("shadow", 0.7), "outside")],
        effects: [shadow(u(3), u(8), 0.6), ...(lit ? [glow(u(8), paint(mk.ink, 0.55), 0)] : [])],
      });
      symbol("Mark", mk.sym, Math.round(size * 0.5), found ? mk.ink : role("inkFaint"));
      if (m.tracked) {
        const k = still() ? 0.5 : (tick % 60) / 60;
        kit.open("Ring", { width: px(size), height: px(size) });
        kit.setDraw({ shape: { kind: "ellipse" }, strokes: [line(u(2), paint(mk.ink, 0.8 * (1 - k)), "outside")] });
        kit.close();
        over({ insets: { start: 0, top: 0 }, inert: true, transform: { scaleX: 1 + k * 0.7, scaleY: 1 + k * 0.7 } });
      }
      kit.close();
      // Its name beside it once zoomed in (the selected one's always), on a dark plate.
      const named = found && (labelA > 0.02 || isSel);
      if (named) {
        kit.row("Label", { padding: xy(u(8), u(3)), part: "WorldMap.label", partClass: "badge" }, () => {
          kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: token("shadow", 0.72) }] });
          text("Text", m.name ?? "", { step: st(14), nowrap: true, font: "display" });
        });
        kit.props({ opacity: isSel ? 1 : labelA });
      }
    });
    if (r.clicked) {
      out.selected = isSel ? "" : key;
      kit.feedback(isSel ? "back" : "select", -1);
    }
  }
  if ((desc.playerX ?? -1) >= 0) {
    const p = at(desc.playerX ?? 0, desc.playerY ?? 0);
    const s = u(30);
    kit.open("Player", { width: px(s), height: px(s), align: "center", cross: "center", part: "WorldMap.player", partClass: "badge" });
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("shadow", 0.55) }], effects: [glow(u(10), token("ink", 0.35), 0)] });
    kit.open("Arrow", { width: px(s), height: px(s) });
    const q = (x: number, y: number) => ({ x: x * s, y: y * s });
    kit.setDraw({ shape: { kind: "path", points: [q(0.5, 0.12), q(0.8, 0.82), q(0.5, 0.64), q(0.2, 0.82)], closed: true, filled: true }, fills: [{ kind: "solid", color: token("ink") }], strokes: [line(1.5, token("shadow", 0.8), "outside")] } as Api.DrawDesc);
    kit.close();
    kit.props({ transform: { rotate: desc.heading ?? 0 } });
    kit.close();
    over({ insets: { start: p.x - s / 2, top: p.y - s / 2 }, inert: true, z: 5 });
    kit.semantics({ role: "image", name: "Player" });
  }
  kit.close();
  over({ insets: { start: 0, top: 0 } });
  kit.close();

  // The side: the legend, or the open place's detail (on a phone: the chips at the top, the detail at the foot).
  const kinds: string[] = [];
  for (const m of markers) if (!kinds.includes(m.kind ?? "poi")) kinds.push(m.kind ?? "poi");
  const sideW = phone ? W - u(16) : Math.min(u(340), Math.round(W * 0.3));
  kit.open("Side", { direction: "column", width: px(sideW), height: phone ? px(H - u(16)) : null, gap: u(12) });
  // On a phone the legend is one row of chips that scrolls sideways, so it never stands over the map in rows. On a
  // desktop an open place's detail takes the side in the legend's place (both would run past the screen's foot).
  const legend = phone || !sel;
  if (phone) {
    kit.openScroll("Legend", { width: grow(), gap: 0, padding: xy(0, 0), axes: "horizontal", part: "WorldMap.legend", partClass: "panel" } as never);
    kit.open("Chips", { direction: "row", gap: u(6) });
  } else if (legend) kit.open("Legend", { direction: "column", width: grow(), gap: u(4), padding: xy(u(14), u(14)), part: "WorldMap.legend", partClass: "panel" });
  if (!phone && legend) {
    kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.9) }, vgrad([[0, token("sheen", 0.06)], [1, token("sheen", 0)]])], effects: [shadow(u(8), u(24), 0.5)] });
    if (desc.title) text("Title", desc.title, { step: st(24), font: "display", heading: true, width: grow() });
  }
  if (legend) for (const k of kinds) {
    const on = !hidden.has(k);
    const mk = kindMark(k);
    const n = markers.filter((m) => (m.kind ?? "poi") === k).length;
    const r = surface(k, {
      part: `WorldMap.entry${flags(!on && "off")}`,
      cls: phone ? "chip" : "listItem",
      fill: phone ? { ...role("shadow"), a: 0.7 } : null,
      washBy: 0.1,
      gap: u(10),
      cross: "center",
      width: phone ? null : grow(),
      padding: xy(u(phone ? 10 : 8), u(phone ? 6 : 7)),
      // A whole touch target under a finger (the phone's chips laid out 36 px tall).
      height: { kind: "fit", min: frame().touch ? 44 : 0 },
      radius: phone ? 999 : u(8),
      semantics: { role: "switch", name: word(desc.kindLabels, k), checked: on },
      props: { states: [{ name: "shown", states: [vstate("on"), vstate("off")], transitions: [] }], state: [{ group: "shown", state: on ? "on" : "off" }] },
    }, () => {
      kit.open("Disc", { width: px(u(24)), height: px(u(24)), align: "center", cross: "center" });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("panelRaised") }], strokes: [line(u(1.5), paint(mk.ink, on ? 1 : 0.35))] });
      symbol("Mark", mk.sym, u(12), on ? mk.ink : role("inkFaint"));
      kit.close();
      text("Name", word(desc.kindLabels, k), { step: st(15), nowrap: true, ellipsis: true, ...(phone ? {} : { width: grow() }), color: on ? "ink" : "inkFaint" });
      if (!phone) text("Count", String(n), { step: st(14), font: "mono", color: "inkMuted", nowrap: true });
    });
    if (r.clicked) {
      out.toggled = k;
      kit.feedback("select", -1);
    }
  }
  if (phone) kit.close();
  if (legend) {
    kit.close();
    kit.semantics({ role: "list", name: desc.title || "Legend" });
  }
  if (phone) kit.spacer("Gap", grow());
  if (sel) {
    const fresh = kit.memory("~w.sel", 0) !== keyHash(sel.key ?? "");
    kit.remember("~w.sel", keyHash(sel.key ?? ""));
    const found = sel.discovered !== false;
    const mk = kindMark(sel.kind ?? "");
    kit.open("Detail", { direction: "column", width: grow(), gap: u(10), padding: xy(u(16), u(16)), part: "WorldMap.detail", partClass: "panel" });
    kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.94) }, vgrad([[0, token("sheen", 0.06)], [1, token("sheen", 0)]])], effects: [shadow(u(10), u(28), 0.55)] });
    if (fresh && !still()) kit.props({ enter: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: u(16) }], transition: spring("panel") } });
    if (sel.image && found && !phone) artSlot("Art", "WorldMap.art", { texture: sel.image, kind: "thumbnail", width: grow(), aspect: 16 / 9, radius: Math.max(4, t.small) });
    kit.row("Head", { width: grow(), gap: u(10), cross: "center" }, () => {
      symbol("Mark", mk.sym, u(18), found ? mk.ink : role("inkFaint"));
      kit.column("Words", { width: grow(), gap: 0 }, () => {
        eyebrow("Kind", word(desc.kindLabels, sel.kind ?? ""));
        text("Name", found ? sel.name ?? "" : desc.undiscoveredLabel ?? "???", { step: st(phone ? 18 : 22), font: "display", heading: true, maxLines: 2, width: grow() });
      });
      const c = surface("Close", { part: "WorldMap.close", cls: "button", fill: null, washBy: 0.14, width: px(Math.max(44, u(32))), height: px(Math.max(44, u(32))), align: "center", cross: "center", radius: 999, semantics: { role: "button", name: desc.closeLabel ?? "Close" } }, () => {
        symbol("Cross", "cross", u(14), role("inkMuted"));
      });
      if (c.clicked) {
        out.selected = "";
        kit.feedback("back", -1);
      }
    });
    if (found && sel.description) text("Description", sel.description, { step: st(15), color: "inkMuted", width: grow(), maxLines: 4 });
    const travel = found && sel.kind === "fastTravel";
    const label = travel ? desc.travelLabel ?? "Fast travel" : sel.tracked ? desc.untrackLabel ?? "Remove waypoint" : desc.trackLabel ?? "Set waypoint";
    const a = btn("Action", label, { variant: travel ? "primary" : "secondary", size: "lg", width: grow(), part: `WorldMap.action${flags(travel ? "travel" : "track")}`, sym: travel ? "bolt" : "pin", ...(travel ? { raised: true } : {}) });
    if (a.clicked) {
      if (travel) out.travel = sel.key ?? null;
      else out.track = sel.key ?? null;
      kit.feedback("confirm", -1);
    }
    kit.close();
    kit.semantics({ role: "region", name: found ? sel.name ?? "" : desc.undiscoveredLabel ?? "???" });
    if (kit.back()) {
      out.selected = "";
      kit.feedback("back", -1);
    }
  }
  kit.close();
  over(phone ? { insets: { start: u(8), top: u(8) }, z: 8 } : { insets: { end: u(20), top: u(20) }, z: 8 });

  kit.close();
  rememberRoom("w", id);
  return out;
}
