// A visual novel's CG gallery and route flowchart (docs/ui.md §11 genre patterns, Appendix B: VN gallery, backlog and
// route flowchart; og.ui.backlog is story.ts's). Read against Steins;Gate's and Clannad's galleries (the pictures as
// tiles by chapter, the unseen ones veiled with how to unlock them, the count seen, the viewer with the versions) and
// Zero Escape's, 428's and Detroit: Become Human's flowcharts (scenes as cards on their columns and lanes, the branches
// with the choices that take them, the endings marked, the current scene ringed, a jump to any scene seen). What has
// been seen, the pictures and the jump are the game's (og.story's save): this draws them and reports what was asked.

import type * as Api from "@origingame/engine";
import {
  artSlot, btn, eyebrow, flags, frame, glow, grow, keyHash, kit, lastInteraction, line, over, px, rememberRoom, role, safeWidth, shadow,
  st, still, surface, symbol, text, token, u, vgrad, xy, type Mut,
} from "./core.ts";
import { chevron, rtl } from "../containers/base.ts";

/** `{n}` and `{total}` filled. */
const fill = (s: string, n: number, total: number): string => s.replace("{n}", String(n)).replace("{total}", String(total));

/** A chapter's tab (a chip on a phone in portrait): its word, a lit underline under the one shown. */
function chapterTab(c: Api.GalleryChapter, on: boolean, portrait: boolean): boolean {
  const { t } = frame();
  const k = c.key ?? "";
  const r = surface(k, {
    part: `CgGallery.tab${flags(on && "selected")}`,
    cls: portrait ? "chip" : "listItem",
    fill: portrait ? (on ? { ...role("primary"), a: 0.24 } : { ...role("shadow"), a: 0.55 }) : null,
    washBy: 0.12,
    direction: "column",
    gap: u(5),
    align: "center",
    cross: "center",
    ...(portrait ? { width: { kind: "fit", min: 44 } as Api.Size, height: { kind: "fit", min: 44 } as Api.Size } : {}),
    padding: xy(u(portrait ? 12 : 14), u(portrait ? 8 : 6)),
    radius: portrait ? 999 : Math.max(4, t.small),
    semantics: { role: "tab", name: c.label ?? k, selected: on },
  }, () => {
    text("Label", c.label ?? k, { step: st(portrait ? 14 : 15), font: "display", nowrap: true, color: on ? "ink" : "inkMuted" });
    if (!portrait) {
      kit.open("Line", { width: grow(), height: px(Math.max(2, u(3))) });
      kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: on ? token("primary") : token("ink", 0) }], ...(on ? { effects: [glow(u(6), token("primary", 0.5), 0)] } : {}) });
      kit.close();
    }
  });
  if (r.clicked && !on) kit.feedback("tab", -1);
  return r.clicked && !on;
}

/** How far a refused element shakes this tick (none under reduced motion). */
function shakeOf(rt: number, rk: number, k: string): number {
  const tick = kit.tick();
  return rt >= 0 && tick - rt < 18 && rk === keyHash(k) && !still() ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.6 : 0;
}

/**
 * A visual novel's CG gallery (Steins;Gate, Clannad): see `fn cgGallery` in crates/og-api/api/ui.ogt.
 */
export function cgGallery(id: string, desc: Api.CgGalleryDesc): Api.CgGalleryResult {
  const out: Mut<Api.CgGalleryResult> = { chapter: null, open: null, refused: null, close: false, step: null, variant: null };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const W = kit.memory(`~cg.w.${id}`, 0) || safeWidth();
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  // Read and written outside the pattern's element: a memory key is the open element's.
  const rt = kit.memory(`~cg.rt.${id}`, -1);
  const rk = kit.memory(`~cg.rk.${id}`, 0);
  const entries = desc.entries ?? [];
  const chapters = desc.chapters ?? [];
  const chapter = desc.chapter ?? "";
  const shown = entries.filter((e) => !chapter || !e.chapter || e.chapter === chapter);
  const viewing = entries.find((e) => e.key === desc.viewing && !e.locked) ?? null;
  const seen = entries.filter((e) => !e.locked).length;

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 10 : 16), part: "CgGallery.frame", partClass: "panel" });

  if (viewing) {
    // The viewer: the picture over the whole box on black, the bar at its foot.
    const versions = [viewing.image ?? null, ...(viewing.variants ?? [])];
    const v = Math.min(desc.variant ?? 0, versions.length - 1);
    const open = shown.filter((e) => !e.locked);
    const at = Math.max(0, open.findIndex((e) => e.key === viewing.key));
    const step = (by: number): void => {
      if (open.length < 2) return;
      out.step = open[(at + by + open.length) % open.length]!.key ?? null;
      kit.feedback("select", -1);
    };
    if (kit.back()) {
      out.close = true;
      kit.feedback("back", -1);
    }
    const lr = (kit.action("right") ? 1 : 0) - (kit.action("left") ? 1 : 0);
    if (lr !== 0) step(rtl() ? -lr : lr);
    const ud = (kit.action("down") ? 1 : 0) - (kit.action("up") ? 1 : 0);
    if (ud !== 0 && versions.length > 1) out.variant = (v + ud + versions.length) % versions.length;
    kit.column("Viewer", { width: grow(), height: grow(), gap: 0, part: "CgGallery.viewer", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow", 0.94) }] });
      artSlot("Picture", "CgGallery.picture", { texture: versions[v] ?? null, kind: "cg", fit: "contain", width: grow(), height: grow(), radius: 0, label: viewing.title ?? "" });
      kit.row("Bar", { width: grow(), gap: u(phone ? 6 : 14), cross: "center", padding: xy(u(phone ? 10 : 24), u(phone ? 8 : 14)), part: "CgGallery.bar", partClass: "panel" }, () => {
        kit.setDraw({ shape: { kind: "rect" }, fills: [vgrad([[0, token("shadow", 0.2)], [1, token("shadow", 0.85)]])] });
        const round = (key: string, label: string, mark: () => void): boolean =>
          surface(key, { part: "CgGallery.button", cls: "button", fill: { ...role("shadow"), a: 0.55 }, washBy: 0.16, width: px(Math.max(44, u(44))), height: px(Math.max(44, u(44))), align: "center", cross: "center", radius: 999, semantics: { role: "button", name: label } }, mark).clicked;
        if (round("Prev", desc.prevLabel ?? "Previous", () => chevron("Mark", "back", u(16), role("ink")))) step(-1);
        kit.column("Words", { width: grow(), gap: u(2) }, () => {
          text("Title", viewing.title ?? "", { step: st(phone ? 16 : 20), font: "display", nowrap: true, ellipsis: true, width: grow() });
          text("Count", `${at + 1} / ${open.length}`, { step: st(13), font: "mono", color: "inkMuted", nowrap: true });
        });
        if (versions.length > 1) {
          kit.row("Pips", { gap: u(2), cross: "center" }, () => {
            versions.forEach((_, i) => {
              const on = i === v;
              const r = surface(String(i), { part: `CgGallery.pip${flags(on && "selected")}`, cls: "button", fill: null, width: px(Math.max(32, u(28))), height: px(Math.max(44, u(32))), align: "center", cross: "center", radius: 999, semantics: { role: "radio", name: `${i + 1}`, checked: on } as never }, () => {
                kit.open("Dot", { width: px(u(on ? 12 : 9)), height: px(u(on ? 12 : 9)) });
                kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: on ? token("primary") : token("ink", 0.45) }], ...(on ? { effects: [glow(u(6), token("primary", 0.6), 0)] } : {}) });
                kit.close();
              });
              if (r.clicked && !on) out.variant = i;
            });
          });
          kit.semantics({ role: "radiogroup", name: viewing.title ?? "" });
        }
        if (round("Next", desc.nextLabel ?? "Next", () => chevron("Mark", "forward", u(16), role("ink")))) step(1);
        if (btn("Close", desc.closeLabel ?? "Close", { variant: "secondary", sym: "cross" }).clicked) {
          out.close = true;
          kit.feedback("back", -1);
        }
      });
      over({ insets: { start: 0, end: 0, bottom: 0 }, z: 2 });
    });
    kit.semantics({ role: "dialog", name: viewing.title ?? "" });
  } else {
    // The tab actions step the chapters, wrapping.
    const st0 = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
    if (st0 !== 0 && chapters.length > 1) {
      const i = Math.max(0, chapters.findIndex((c) => c.key === chapter));
      out.chapter = chapters[(i + st0 + chapters.length) % chapters.length]!.key ?? null;
      kit.feedback("tab", -1);
    }
    kit.row("Head", { width: grow(), gap: u(16), cross: "center" }, () => {
      if (chapters.length > 1) {
        kit.openScroll("Tabs", { width: grow(), gap: 0, padding: xy(0, 0), axes: "horizontal" } as never);
        kit.open("Row", { direction: "row", gap: u(portrait ? 6 : 4), cross: "center" });
        for (const c of chapters) if (chapterTab(c, (c.key ?? "") === chapter, portrait)) out.chapter = c.key ?? "";
        kit.close();
        kit.close();
        kit.semantics({ role: "tablist", name: desc.title || "Gallery" });
      } else kit.spacer("Gap", grow());
      kit.row("Count", { gap: u(8), cross: "center", padding: xy(u(12), u(6)), part: "CgGallery.count", partClass: "chip" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.55) }] });
        symbol("Eye", "eye", u(16), role("primary"), { icon: "eye" });
        text("Text", fill(desc.countLabel ?? "{n} / {total}", seen, entries.length), { step: st(15), font: "mono", nowrap: true });
      });
    });
    // The grid: whole rows of 16:9 tiles, each its title in its own row under it.
    const gap = u(phone ? 8 : 16);
    const want = u(portrait ? 160 : 260) * Math.min(1.3, ts);
    const cols = Math.max(portrait ? 2 : 2, Math.floor((W - u(8) + gap) / (want + gap)));
    const cw = Math.floor((W - u(8) - gap * (cols - 1)) / cols);
    const chH = Math.round((cw * 9) / 16);
    const short = chH < u(phone ? 56 : 60) + 2 * u(phone ? 12 : 13) * 1.35 * ts;
    kit.openScroll("Grid", { width: grow(), height: grow(), gap, padding: xy(u(4), u(4)), fade: 32, fadeInto: "shadow" } as never);
    for (let r0 = 0; r0 < shown.length; r0 += cols) {
      kit.row(kit.idi("Line", r0 / cols), { gap }, () => {
        for (const e of shown.slice(r0, r0 + cols)) {
          const k = e.key ?? "";
          const locked = !!e.locked;
          const rr = Math.max(4, t.small);
          const r = surface(k, {
            part: `CgGallery.tile${flags(locked && "locked", e.fresh && "fresh")}`,
            cls: "listItem",
            fill: null,
            washBy: 0.04,
            direction: "column",
            gap: u(6),
            width: px(cw),
            lift: locked ? 0 : u(3),
            semantics: { role: "button", name: locked ? [desc.lockedLabel ?? "???", e.hint ?? ""].filter(Boolean).join(", ") : [e.title ?? k, e.fresh ? desc.newLabel ?? "New" : ""].filter(Boolean).join(", ") },
            props: { ...(shakeOf(rt, rk, k) ? { transform: { x: shakeOf(rt, rk, k) } } : {}) },
          }, () => {
            const ix = lastInteraction(k);
            // The focus ring and lift shadow are the picture's own: setDraw after it would replace its draw (it is the element added last).
            const lit = ix.hovered || ix.focused;
            artSlot("Picture", "CgGallery.picture", { texture: locked ? null : e.image ?? null, kind: "cg", width: px(cw), height: px(chH), radius: rr, label: "", strokes: lit ? [line(u(2), token("focus"), "outside")] : [], effects: !locked && lit ? [shadow(u(6), u(18), 0.5)] : [] }, () => {
              if (locked) {
                kit.column("Veil", { width: grow(), height: grow(), gap: u(6), align: "center", cross: "center", padding: xy(u(10), u(8)), part: "CgGallery.veil", partClass: "panel" }, () => {
                  kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [vgrad([[0, token("well", 0.94)], [1, token("shadow", 0.96)]])], strokes: [line(1, token("ink", 0.1))] });
                  symbol("Lock", "lock", u(phone ? 22 : 28), role("inkMuted"), { icon: "lock" });
                  if (e.hint && !short) text("Hint", e.hint, { step: st(phone ? 12 : 13), color: "inkMuted", align: "center", width: grow(), maxLines: 2 });
                });
                over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
              }
              if (e.fresh && !locked) {
                kit.row("New", { padding: xy(u(8), u(2)), part: "CgGallery.new", partClass: "badge" }, () => {
                  kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("primary") }], effects: [glow(u(6), token("primary", 0.5), 0)] });
                  text("Text", (desc.newLabel ?? "New").toLocaleUpperCase(), { step: st(11), font: "display", nowrap: true, color: "onPrimary" });
                });
                over({ insets: { end: u(6), top: u(6) }, inert: true });
              }
            });
            text("Title", locked ? desc.lockedLabel ?? "???" : e.title ?? k, { step: st(phone ? 13 : 15), nowrap: true, ellipsis: true, width: grow(), color: locked ? "inkFaint" : "ink" });
            // A veil too short for the lock and two lines of the hint: the hint stands in its own row under the title.
            if (locked && short && e.hint) text("Hint", e.hint, { step: st(12), color: "inkMuted", width: grow(), maxLines: 2 });
          });
          if (r.clicked) {
            if (locked) {
              out.refused = k;
              kit.feedback("error", -1);
            } else {
              out.open = k;
              kit.feedback("select", -1);
            }
          }
        }
      });
    }
    kit.close();
    kit.semantics({ role: "grid", name: desc.title || "Gallery" });
  }
  kit.close();
  rememberRoom("cg", id);
  kit.remember(`~cg.rt.${id}`, out.refused ? tick : rt);
  kit.remember(`~cg.rk.${id}`, out.refused ? keyHash(out.refused) : rk);
  return out;
}

/**
 * A visual novel's route flowchart (Zero Escape, 428, Detroit: Become Human): see `fn routeChart` in
 * crates/og-api/api/ui.ogt.
 */
export function routeChart(id: string, desc: Api.RouteChartDesc): Api.RouteChartResult {
  const out: Mut<Api.RouteChartResult> = { selected: null, jump: null, refused: null };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const rt = kit.memory(`~rc.rt.${id}`, -1);
  const rk = kit.memory(`~rc.rk.${id}`, 0);
  const scenes = desc.scenes ?? [];
  const links = desc.links ?? [];
  const chapters = desc.chapters ?? [];
  const sel = scenes.find((s) => s.key === desc.selected) ?? null;
  const flip = rtl();

  // The grid: a card per cell, the columns' gap wide enough for a choice's words.
  const cw = Math.round(u(phone ? 128 : 168) * Math.min(1.3, ts));
  const thumbH = Math.round((cw * 9) / 16);
  const titleH = Math.round(u(phone ? 34 : 38) * Math.min(1.6, ts));
  const ch = thumbH + titleH;
  const colGap = Math.round(u(phone ? 72 : 112) * Math.min(1.3, ts));
  const laneGap = Math.round(u(phone ? 28 : 40) * Math.min(1.3, ts));
  const headH = chapters.length > 0 ? Math.round(u(32) * Math.min(1.5, ts)) : 0;
  const pad = u(phone ? 12 : 24);
  const minLane = Math.min(0, ...scenes.map((s) => s.lane ?? 0));
  const cols = Math.max(chapters.length, ...scenes.map((s) => (s.column ?? 0) + 1), 1);
  const lanes = Math.max(1, ...scenes.map((s) => (s.lane ?? 0) - minLane + 1));
  const mapW = pad * 2 + cols * cw + (cols - 1) * colGap;
  const mapH = pad * 2 + headH + lanes * ch + (lanes - 1) * laneGap;
  // A cell's place in the map, from its start edge (logical: the map is mirrored right to left).
  const at = (s: Api.RouteScene) => ({ x: pad + (s.column ?? 0) * (cw + colGap), y: pad + headH + ((s.lane ?? 0) - minLane) * (ch + laneGap) });
  const byKey = new Map(scenes.map((s) => [s.key ?? "", s]));

  kit.open(id, { direction: portrait ? "column" : "row", width: grow(), height: grow(), gap: u(phone ? 10 : 16), part: "RouteChart.frame", partClass: "panel" });

  kit.column("Chart", { width: grow(), height: grow(), gap: u(8) }, () => {
    kit.row("Head", { width: grow(), gap: u(12), cross: "center" }, () => {
      eyebrow("Title", desc.title ?? "");
      kit.spacer("Gap", grow());
      text("Count", fill(desc.seenLabel ?? "{n} / {total} scenes", scenes.filter((s) => s.seen).length, scenes.length), { step: st(14), font: "mono", nowrap: true, color: "inkMuted" });
    });
    kit.openScroll("View", { width: grow(), height: grow(), gap: 0, padding: xy(0, 0), axes: "both", fade: 24, fadeInto: "shadow", part: "RouteChart.view" } as never);
    kit.open("Map", { width: px(mapW), height: px(mapH) });
    kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.55) }] });
    // The chapters' headings over their columns.
    chapters.forEach((c, i) => {
      kit.row(`${c.key ?? i}~Heading`, { width: px(cw), height: px(headH), align: "center", cross: "center", part: "RouteChart.heading", partClass: "badge" }, () => {
        text("Text", (c.label ?? "").toLocaleUpperCase(), { step: st(12), font: "display", nowrap: true, ellipsis: true, color: "inkMuted" });
      });
      over({ insets: { start: pad + i * (cw + colGap), top: pad - u(6) }, inert: true });
    });
    // The branches: an elbow from a card's end edge to the next card's start edge, the choice's words at the fork.
    links.forEach((l, i) => {
      const a = byKey.get(l.from ?? "");
      const b = byKey.get(l.to ?? "");
      if (!a || !b) return;
      const pa = at(a);
      const pb = at(b);
      const x0 = pa.x + cw;
      const y0 = pa.y + thumbH / 2;
      const x1 = pb.x;
      const y1 = pb.y + thumbH / 2;
      const mid = x0 + Math.max(u(16), (x1 - x0) * 0.35);
      const lx = Math.min(x0, x1);
      const ly = Math.min(y0, y1) - 2;
      const lw = Math.max(4, Math.abs(x1 - x0));
      const lh = Math.max(4, Math.abs(y1 - y0) + 4);
      // Points in the box, mirrored right to left (insets are logical, a path's points physical).
      const P = (x: number, y: number) => ({ x: flip ? lw - (x - lx) : x - lx, y: y - ly });
      const lit = !!l.taken;
      kit.open(kit.idi("Link", i), { width: px(lw), height: px(lh), part: `RouteChart.link${flags(lit && "taken")}`, partClass: "badge" });
      kit.setDraw({
        shape: { kind: "path", points: [P(x0, y0), P(mid, y0), P(mid, y1), P(x1, y1)], width: Math.max(1.5, u(lit ? 2.5 : 1.5)), cap: "round", join: "round", closed: false },
        fills: [{ kind: "solid", color: lit ? token("primary", 0.9) : token("ink", 0.28) }],
        ...(lit ? { effects: [glow(u(5), token("primary", 0.45), 0)] } : {}),
      } as Api.DrawDesc);
      kit.close();
      over({ insets: { start: lx, top: ly }, inert: true });
      if (l.label) {
        kit.row(kit.idi("Choice", i), { padding: xy(u(8), u(3)), part: "RouteChart.choice", partClass: "badge" }, () => {
          kit.setDraw({ shape: { kind: "rect", radii: [u(10), u(10), u(10), u(10)] }, fills: [{ kind: "solid", color: token("shadow", 0.85) }], strokes: [line(1, lit ? token("primary", 0.7) : token("ink", 0.18))] });
          text("Text", l.label!, { step: st(12), nowrap: true, ellipsis: true, color: lit ? "ink" : "inkMuted", width: { kind: "fit", max: colGap + cw * 0.6 } as Api.Size });
        });
        over({ insets: { start: mid + u(6), top: y1 - (y1 >= y0 ? u(26) * Math.min(1.5, ts) : -u(6)) }, inert: true, z: 3 });
      }
    });
    // The scenes.
    let curPath = "";
    for (const s of scenes) {
      const k = s.key ?? "";
      const p = at(s);
      const locked = !!s.locked;
      const on = !!sel && sel.key === k;
      const rr = Math.max(4, t.small);
      const end = s.ending ? (s.good ? role("success") : role("danger")) : null;
      const sh = shakeOf(rt, rk, k);
      const r = surface(k, {
        part: `RouteChart.scene${flags(!!s.seen && "seen", locked && "locked", !!s.current && "current", !!s.ending && "ending")}`,
        cls: "listItem",
        fill: null,
        washBy: 0.04,
        direction: "column",
        gap: 0,
        width: px(cw),
        height: px(ch),
        radius: rr,
        semantics: { role: "button", name: [locked ? desc.lockedLabel ?? "???" : s.title ?? k, s.current ? desc.currentLabel ?? "Now" : "", s.ending ?? ""].filter(Boolean).join(", "), selected: on },
        layer: { insets: { start: p.x, top: p.y }, z: on ? 5 : 2, ...(sh ? { transform: { x: sh } } : {}) },
      }, () => {
        const ix = lastInteraction(k);
        kit.setDraw({
          shape: { kind: "rect", radii: [rr, rr, rr, rr] },
          fills: [{ kind: "solid", color: token("panel", s.seen ? 0.95 : 0.7) }],
          strokes: [s.current ? line(u(2.5), token("focus"), "outside") : on ? line(u(2), token("primary"), "outside") : ix.hovered || ix.focused ? line(u(1.5), token("ink", 0.45), "outside") : line(1, end ? { ...end, a: 0.7 } as never : token("ink", s.seen ? 0.2 : 0.08))],
          effects: [shadow(u(4), u(12), 0.45), ...(s.current ? [glow(u(10), token("focus", 0.45), 0)] : [])],
          clip: "shape",
        } as Api.DrawDesc);
        artSlot("Thumb", "RouteChart.thumb", { texture: locked ? null : s.image ?? null, kind: "cg", width: px(cw), height: px(thumbH), radius: [rr, rr, 0, 0], label: "" }, () => {
          if (locked || !s.seen) {
            kit.column("Veil", { width: grow(), height: grow(), align: "center", cross: "center" }, () => {
              kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow", locked ? 0.86 : 0.45) }] });
              if (locked) symbol("Lock", "lock", u(18), role("inkMuted"), { icon: "lock" });
            });
            over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
          }
          if (s.current) {
            kit.row("Now", { padding: xy(u(7), u(2)), part: "RouteChart.now", partClass: "badge" }, () => {
              kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("focus") }] });
              text("Text", (desc.currentLabel ?? "Now").toLocaleUpperCase(), { step: st(11), font: "display", nowrap: true, color: "onPrimary" });
            });
            over({ insets: { start: u(5), top: u(5) }, inert: true });
          }
        });
        kit.row("Words", { width: grow(), height: grow(), gap: u(6), cross: "center", padding: xy(u(8), 0) }, () => {
          if (end) {
            kit.open("Band", { width: px(Math.max(3, u(4))), height: px(Math.round(titleH * 0.6)) });
            kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: { ...end, a: 1 } as never }] });
            kit.close();
          }
          text("Title", locked ? desc.lockedLabel ?? "???" : s.title ?? k, { step: st(phone ? 12 : 13), nowrap: true, ellipsis: true, width: grow(), color: s.seen ? "ink" : "inkMuted" });
        });
      });
      if (s.current) curPath = kit.lastPath();
      if (r.clicked) {
        if (locked) {
          out.refused = k;
          kit.feedback("error", -1);
        } else if (!on) {
          out.selected = k;
          kit.feedback("select", -1);
        }
      }
    }
    kit.close();
    kit.close();
    kit.semantics({ role: "group", name: desc.title || "Routes" });
    // Open on the current scene, centred, until it shows; then left to the player's scrolling.
    const want = keyHash(curPath);
    if (curPath && kit.memory(`~rc.shown.${id}`, 0) !== want) {
      kit.scrollIntoView(curPath, true);
      if (kit.bounds(curPath)) kit.remember(`~rc.shown.${id}`, want);
    }
  });

  // The selected scene's detail: its thumbnail large, its title and ending, Jump here.
  if (sel) {
    const dw = portrait ? null : Math.round(Math.min(u(340), safeWidth() * 0.26) * Math.min(1.25, ts));
    kit.column("Detail", { width: dw === null ? grow() : px(dw), gap: u(10), padding: xy(u(14), u(14)), part: "RouteChart.detail", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.92) }], strokes: [line(1, token("ink", 0.12))], effects: [shadow(u(6), u(20), 0.45)] });
      if (portrait) {
        kit.row("Body", { width: grow(), gap: u(12), cross: "center" }, () => {
          artSlot("Thumb", "RouteChart.detailThumb", { texture: sel.image ?? null, kind: "cg", width: px(u(120)), height: px(Math.round((u(120) * 9) / 16)), radius: Math.max(4, t.small), label: "" });
          kit.column("Words", { width: grow(), gap: u(4) }, () => {
            text("Title", sel.title ?? "", { step: st(16), font: "display", heading: true, maxLines: 2, width: grow() });
            if (sel.ending) text("Ending", sel.ending, { step: st(13), font: "display", nowrap: true, color: sel.good ? "success" : "danger" });
          });
        });
      } else {
        artSlot("Thumb", "RouteChart.detailThumb", { texture: sel.image ?? null, kind: "cg", width: grow(), aspect: 16 / 9, radius: Math.max(4, t.small), label: "" });
        text("Title", sel.title ?? "", { step: st(20), font: "display", heading: true, maxLines: 2, width: grow() });
        if (sel.ending) text("Ending", sel.ending.toLocaleUpperCase(), { step: st(14), font: "display", nowrap: true, color: sel.good ? "success" : "danger" });
      }
      if (sel.seen && btn("Jump", desc.jumpLabel ?? "Jump here", { variant: "primary", size: "lg", width: grow(), raised: true }).clicked) {
        out.jump = sel.key ?? null;
        kit.feedback("confirm", -1);
      }
    });
    kit.semantics({ role: "region", name: sel.title ?? "" });
  }

  kit.close();
  rememberRoom("rc", id);
  kit.remember(`~rc.rt.${id}`, out.refused ? tick : rt);
  kit.remember(`~rc.rk.${id}`, out.refused ? keyHash(out.refused) : rk);
  return out;
}
