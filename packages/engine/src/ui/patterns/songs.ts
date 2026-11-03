// The song select (docs/ui.md §11 genre patterns, Appendix B: song and
// track select with preview). Read against Project Sekai's and osu!'s
// (the songs on an angled wheel at the side, the selected one large with
// its jacket, the charts as chips with their levels, the best score and
// grade) and Beat Saber's (the preview's waveform under the cover). The
// songs, the selection, the chart and the preview's sound are the game's;
// the wheel's glide is the rows' own motion.

import type * as Api from "@origingame/engine";
import {
  firstRoom,
  artSlot, btn, clamp, eyebrow, flags, frame, glow, grouped, grow, keyHash, kit, lastInteraction, line, mix, over, paint, px, recallPath, rememberPath, role,
  rememberRoom, safeHeight, safeWidth,
  shadow, spring, st, still, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut,
} from "./core.ts";
import { cornerTag } from "./rewards.ts";

/** Seconds as m:ss. */
function mmss(s: number): string {
  const n = Math.max(0, Math.floor(s));
  return `${Math.floor(n / 60)}:${String(n % 60).padStart(2, "0")}`;
}

/** The accuracy as a ring: an arc of it in the ink, the share in tabular numerals in the middle and the caption under it. */
function accuracyRing(id: string, f: number, size: number, caption: string): void {
  const thick = Math.max(4, Math.round(size * 0.08));
  kit.open(id, { width: px(size), height: px(size), align: "center", cross: "center", part: "Songs.accuracy", partClass: "badge" });
  kit.setDraw({ shape: { kind: "arc", thickness: thick }, fills: [{ kind: "solid", color: token("ink", 0.12) }] });
  kit.open("Fill", { width: px(size), height: px(size) });
  kit.setDraw({ shape: { kind: "arc", start: -Math.PI / 2, sweep: Math.max(0.0001, clamp(f, 0, 1)) * Math.PI * 2, thickness: thick, cap: "round" }, fills: [{ kind: "solid", color: token("ink") }], effects: [glow(u(6), token("ink", 0.35), 0)] });
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true });
  text("Value", `${(clamp(f, 0, 1) * 100).toFixed(1)}%`, { step: st(Math.round(size * 0.2)), font: "mono", nowrap: true });
  text("Caption", caption.toLocaleUpperCase(), { step: "caption", font: "display", color: "inkMuted", nowrap: true });
  kit.close();
  kit.semantics({ role: "status", name: `${caption} ${(clamp(f, 0, 1) * 100).toFixed(1)}%` });
}

/**
 * A song select (Project Sekai, osu!, Beat Saber): see `fn songSelect` in
 * crates/og-api/api/ui.ogt.
 */
export function songSelect(id: string, desc: Api.SongSelectDesc): Api.SongSelectResult {
  const out: Mut<Api.SongSelectResult> = { selected: null, difficulty: null, play: false, preview: false, refused: null };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const songs = desc.songs ?? [];
  const si = Math.max(0, songs.findIndex((s) => s.key === desc.selected));
  const song = songs[si] ?? null;
  const charts = song?.difficulties ?? [];
  const chart = charts.find((c) => c.key === desc.difficulty) ?? charts[0] ?? null;
  const W = kit.memory(`~g.w.${id}`, 0) || firstRoom().w;
  const H = kit.memory(`~g.h.${id}`, 0) || firstRoom().h;
  const rt = kit.memory("~g.rt", 0);
  const rk = kit.memory("~g.rk", 0);
  const songHash = keyHash(song?.key ?? "");
  const changed = kit.memory("~g.song", 0) !== songHash;
  kit.remember("~g.song", songHash);
  const fade: Api.ElementProps = changed && !still() ? { enter: { values: [{ prop: "opacity", value: 0 }], transition: spring("ui") } } : {};
  const prev = desc.preview ?? {};

  kit.open(id, { direction: portrait ? "column" : "row", width: grow(), height: grow(), gap: u(phone ? 12 : 28), part: "Songs.frame", partClass: "panel" });

  // The selected song: its jacket and preview, then its detail (beside them; under them on a phone in portrait).
  const jacket = portrait ? u(112) : Math.round(Math.min(H * 0.56, W * 0.28));
  const previewBar = (bw: number): void => {
    const seconds = song?.seconds ?? 0;
    const f = seconds > 0 ? clamp((prev.position ?? 0) / seconds, 0, 1) : 0;
    kit.row("Preview", { width: px(bw), gap: u(14), cross: "center", padding: xy(u(12), u(10)), part: `Songs.preview${flags(prev.playing && "playing")}`, partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("shadow", 0.55) }, vgrad([[0, token("sheen", 0.06)], [1, token("sheen", 0)]])], strokes: [line(1, token("ink", 0.08))] });
      const r = surface("Toggle", { part: "Songs.toggle", cls: "button", fill: { ...role("ink"), a: 0.1 }, washBy: 0.14, width: px(u(44)), height: px(u(44)), align: "center", cross: "center", radius: 999, semantics: { role: "button", name: `${desc.previewLabel ?? "Preview"} ${mmss(prev.position ?? 0)}`, ...(prev.playing ? { selected: true } : {}) } }, () => {
        kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("ink", 0.1) }], strokes: [line(u(1.5), token("ink", 0.7))] });
        const s = u(16);
        kit.open("Mark", { width: px(s), height: px(s) });
        const q = (x: number, y: number) => ({ x: x * s, y: y * s });
        // Playing: two bars (pause); else a triangle (play), its weight nudged right to sit centred.
        if (prev.playing) kit.setDraw({ shape: { kind: "path", points: [q(0.3, 0.15), q(0.3, 0.85), q(0.3, 0.5), q(0.7, 0.5), q(0.7, 0.15), q(0.7, 0.85)], width: Math.max(2, u(3)), cap: "round" }, fills: [{ kind: "solid", color: token("ink") }] } as Api.DrawDesc);
        else kit.setDraw({ shape: { kind: "path", points: [q(0.28, 0.12), q(0.88, 0.5), q(0.28, 0.88)], closed: true, filled: true }, fills: [{ kind: "solid", color: token("ink") }] } as Api.DrawDesc);
        kit.close();
      });
      if (r.clicked) {
        out.preview = true;
        kit.feedback("toggle", -1);
      }
      // The waveform: a bar per peak, lit up to where the preview is.
      const peaks = prev.peaks ?? [];
      // The waveform takes the bar's width; the time sits under it, so nothing beside the bar can run into it.
      const waveW = bw - u(44) - u(14) - u(24);
      const n = Math.max(1, peaks.length);
      const bwid = Math.max(1, Math.floor(waveW / n) - 1);
      const wh = u(30);
      kit.open("Track", { direction: "column", width: px(waveW), gap: u(4), cross: "end" });
      kit.row("Wave", { width: px(waveW), height: px(wh), gap: Math.max(1, Math.floor(waveW / n) - bwid), cross: "center", part: "Songs.wave", partClass: "badge" }, () => {
        if (peaks.length === 0) {
          kit.open("Flat", { width: grow(), height: px(2), background: token("ink", 0.25) });
          kit.close();
        }
        peaks.forEach((p, i) => {
          const lit = (i + 0.5) / n <= f;
          kit.open(kit.idi("B", i), { width: px(bwid), height: px(Math.max(2, Math.round(wh * clamp(p, 0.04, 1)))), radius: 1, background: lit ? token("ink") : token("ink", 0.28) });
          kit.close();
        });
      });
      text("Time", `${mmss(prev.position ?? 0)} / ${mmss(seconds)}`, { step: st(12), font: "mono", color: "inkMuted", nowrap: true });
      kit.close();
    });
  };

  const left = (): void => {
    kit.column("Left", { width: px(jacket), gap: u(16) }, () => {
      artSlot("Jacket", "Songs.jacket", { texture: song?.jacket ?? null, kind: "jacket", width: px(jacket), height: px(jacket), radius: Math.max(6, t.panel), focal: { x: 0.5, y: 0.5 } });
      kit.props(fade);
    });
  };

  const playable = !!song && !song.locked && !!chart && !chart.locked;
  const play = (w: Api.Size): void => {
    const r = btn("Play", desc.playLabel ?? "Play", { variant: "primary", size: "lg", width: w, disabled: !playable, part: "Songs.play", raised: true });
    if (r.clicked && playable) {
      out.play = true;
      kit.feedback("confirm", -1);
    }
  };
  // The detail; compact (a phone in portrait), its head and stats are in the top row and its preview and Play at the foot.
  const detail = (compact: boolean): void => {
    // Beside the jacket it scrolls in the room the foot leaves it (at 200 % text its words outgrow it).
    if (compact) kit.open("Detail", { direction: "column", width: grow(), gap: u(phone ? 10 : 18), part: "Songs.detail", partClass: "panel" });
    else kit.openScroll("Detail", { width: grow(), height: grow(), gap: u(18), padding: xy(0, 0), fade: 40, fadeInto: "shadow", part: "Songs.detail" } as never);
    if (!compact) kit.column("Head", { width: grow(), gap: u(4) }, () => {
      text("Title", song?.title ?? "", { step: phone ? "xl3" : "displayMd", font: "display", heading: true, maxLines: 2, width: grow() });
      kit.props(fade);
      text("Artist", song?.artist ?? "", { step: st(phone ? 15 : 20), color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
    });
    if (!compact) {
      kit.row("Stats", { gap: u(28), cross: "end", part: "Songs.stats", partClass: "badge" }, () => {
        if ((song?.bpm ?? 0) > 0) kit.column("Bpm", { gap: 0 }, () => {
          eyebrow("Label", desc.bpmLabel ?? "BPM");
          text("Value", String(Math.round(song!.bpm!)), { step: st(30), font: "mono", nowrap: true });
        });
        if ((song?.seconds ?? 0) > 0) kit.column("Length", { gap: 0 }, () => {
          eyebrow("Label", desc.lengthLabel ?? "Length");
          text("Value", mmss(song!.seconds!), { step: st(30), font: "mono", nowrap: true });
        });
      });
    }
    // The charts: a chip each, its name in caps over its level; the selected lit with a rim and a bar under it.
    // One row of chips sharing the width (a chart never wraps under its neighbours).
    kit.open("Charts", { direction: "row", gap: u(phone ? 6 : 10), width: grow() });
    charts.forEach((c) => {
      const on = !!chart && chart.key === c.key;
      const ix = lastInteraction(c.key ?? "");
      const r = surface(c.key ?? "", {
        part: `Songs.chip${flags(on && "selected", c.locked && "locked")}`,
        cls: "chip",
        fill: null,
        direction: "column",
        width: grow(),
        padding: xy(u(8), u(phone ? 6 : 10)),
        gap: 0,
        cross: "center",
        radius: Math.max(4, t.small),
        semantics: { role: "radio", name: `${c.name ?? c.key} ${c.level ?? 0}`, checked: on, disabled: !!c.locked },
        props: { states: [{ name: "chart", states: [vstate("open"), vstate("selected"), vstate("locked")], transitions: [] }], state: [{ group: "chart", state: c.locked ? "locked" : on ? "selected" : "open" }] },
      }, () => {
        const rr = Math.max(4, t.small);
        kit.setDraw({
          shape: { kind: "rect", radii: [rr, rr, rr, rr] },
          fills: [on ? vgrad([[0, token("panelRaised")], [1, paint(mix(role("panelRaised"), role("ink"), 0.08))]]) : { kind: "solid", color: token("shadow", 0.5) }],
          strokes: on ? [line(u(2), token("focus"))] : ix.hovered || ix.focused ? [line(u(1.5), token("ink", 0.4))] : [line(1, token("ink", 0.12))],
          effects: on ? [glow(u(7), token("focus", 0.4), 0), shadow(u(3), u(10), 0.4)] : [],
        });
        text("Name", (c.name ?? c.key ?? "").toLocaleUpperCase(), { step: st(phone ? 11 : 13), font: "display", nowrap: true, ellipsis: true, color: on ? "ink" : "inkMuted" });
        kit.row("Level", { gap: u(4), cross: "center" }, () => {
          if (c.locked) symbol("Lock", "lock", u(14), role("inkFaint"), { icon: "lock" });
          text("Value", String(c.level ?? 0), { step: st(phone ? 20 : 28), font: "mono", nowrap: true, color: c.locked ? "inkFaint" : "ink" });
        });
      });
      if (r.clicked && !on) {
        if (c.locked) {
          out.refused = c.key ?? null;
          kit.feedback("error", -1);
        } else {
          out.difficulty = c.key ?? null;
          kit.feedback("tab", -1);
        }
      }
    });
    kit.close();
    kit.semantics({ role: "radiogroup", name: song?.title ?? "" });
    // The best (on a phone in portrait it is a line in the head, so the wheel keeps its room): the score in tabular numerals, the grade in the display face, the full combo badge, the accuracy ring.
    if (!compact) kit.row("Best", { width: grow(), gap: u(phone ? 14 : 28), cross: "center", wrap: true, part: "Songs.score", partClass: "badge" }, () => {
      kit.column("Score", { gap: 0 }, () => {
        eyebrow("Label", desc.bestLabel ?? "Best score");
        text("Value", (song?.best ?? 0) > 0 ? grouped(song!.best!) : "—", { step: st(phone ? 24 : 40), font: "mono", nowrap: true });
      });
      if (song?.grade) {
        kit.column("Grade", { gap: 0, part: "Songs.grade", partClass: "badge" }, () => {
          eyebrow("Label", desc.gradeLabel ?? "Grade");
          text("Value", song.grade!, { step: st(phone ? 28 : 44), font: "display", nowrap: true, color: /^S/.test(song.grade!) ? "gold" : "ink" });
        });
      }
      if (song?.fullCombo) {
        kit.row("Combo", { padding: xy(u(12), u(5)), part: "Songs.combo", partClass: "badge" }, () => {
          kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, strokes: [line(u(1.5), token("gold"))], fills: [{ kind: "solid", color: token("gold", 0.12) }], effects: [glow(u(6), token("gold", 0.35), 0)] });
          text("Text", (desc.fullComboLabel ?? "Full combo").toLocaleUpperCase(), { step: st(13), font: "display", color: "gold", nowrap: true });
        });
      }
    });
    if (!compact) kit.semantics({ role: "status", name: [desc.bestLabel ?? "Best score", (song?.best ?? 0) > 0 ? grouped(song!.best!) : "", song?.grade ?? "", song?.fullCombo ? desc.fullComboLabel ?? "Full combo" : ""].filter(Boolean).join(" ") });
    kit.close();
  };

  if (portrait) {
    kit.row("Top", { width: grow(), gap: u(14), cross: "center" }, () => {
      left();
      kit.column("Side", { width: grow(), gap: u(4) }, () => {
        text("Title", song?.title ?? "", { step: "xl3", font: "display", heading: true, maxLines: 2, width: grow() });
        kit.props(fade);
        text("Artist", song?.artist ?? "", { step: st(15), color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
        if ((song?.bpm ?? 0) > 0) text("Stats", `${desc.bpmLabel ?? "BPM"} ${Math.round(song!.bpm!)} · ${mmss(song?.seconds ?? 0)}`, { step: st(13), font: "mono", color: "inkMuted", nowrap: true });
        if ((song?.best ?? 0) > 0) text("Best", [`${desc.bestLabel ?? "Best score"} ${grouped(song!.best!)}`, song?.grade ?? ""].filter(Boolean).join(" · "), { step: st(13), font: "mono", nowrap: true, ellipsis: true, width: grow() });
      });
    });
  }
  if (!portrait) {
    // The jacket and the detail side by side, the preview's bar, the accuracy ring and Play across their foot.
    kit.column("Main", { width: grow(), height: grow(), gap: u(24) }, () => {
      kit.row("Top", { width: grow(), height: grow(), gap: u(28) }, () => {
        left();
        detail(false);
      });
      kit.row("Foot", { width: grow(), gap: u(16), cross: "center" }, () => {
        const ring = (song?.accuracy ?? -1) >= 0 ? u(96) : 0;
        const mw = W - Math.round(W * 0.32) - u(28);
        previewBar(Math.max(u(240), mw - u(16) - u(220) - (ring ? ring + u(16) : 0)));
        if (ring) accuracyRing("Accuracy", song!.accuracy!, ring, desc.accuracyLabel ?? "Accuracy");
        play(px(u(220)));
      });
    });
  } else detail(true);

  // The wheel: the rows around the selected one, leaning away and fading as they go; gliding as the selection moves.
  const ww = portrait ? W : Math.round(W * 0.32);
  // The wheel grows into the room the column leaves it; its rows are placed by the height it had last tick.
  // Before it has a height, a guess short of the room (a row built past the edge would stand off the screen for a tick).
  const room = portrait ? Math.max(u(200), H - jacket - u(260)) : H;
  const laidWheel = kit.bounds(recallPath(`${id}~wheel`))?.height ?? 0;
  const wh = laidWheel > 0 ? Math.min(laidWheel, room) : Math.round(room * 0.8);
  const rowH = u(phone ? 64 : 84);
  const big = Math.round(rowH * (portrait ? 1 : 1.22));
  const gap = u(phone ? 6 : 10);
  // Whole rows only: a row the wheel's edge would cut stays unbuilt (the wheel does not clip, so a cut row would stand
  // over the head or the detail).
  const reach = Math.max(0, Math.floor((wh / 2 - big / 2) / (rowH + gap)));
  kit.open("Wheel", { width: px(ww), height: grow(), part: "Songs.wheel", partClass: "panel" });
  for (let k = Math.max(0, si - reach); k <= Math.min(songs.length - 1, si + reach); k++) {
    const s = songs[k]!;
    const off = k - si;
    const key = s.key ?? `s${k}`;
    const sel = off === 0;
    const h = sel ? big : rowH;
    const y = Math.round(wh / 2 + (off === 0 ? -big / 2 : off > 0 ? big / 2 + gap + (off - 1) * (rowH + gap) : -big / 2 - gap - (-off) * (rowH + gap) + 0));
    const lean = portrait ? 0 : Math.round(Math.pow(Math.abs(off), 1.25) * u(18));
    const w = ww - lean - (sel ? 0 : u(12));
    const ix = lastInteraction(key);
    const shaking = rt > 0 && tick - rt < 18 && rk === keyHash(key) && !still() ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.5 : 0;
    const r = surface(key, {
      part: "Songs.row",
      cls: "listItem",
      flags: [sel ? "selected" : "", s.locked ? "locked" : "", s.isNew ? "new" : ""],
      fill: null,
      direction: "row",
      width: px(w),
      height: px(h),
      gap: u(12),
      cross: "center",
      padding: { left: u(8), right: u(16), top: u(8), bottom: u(8) },
      radius: Math.max(6, t.panel),
      semantics: { role: "option", name: [s.title ?? key, s.artist ?? "", (s.bpm ?? 0) > 0 ? `${desc.bpmLabel ?? "BPM"} ${Math.round(s.bpm!)}` : "", s.locked ? "locked" : ""].filter(Boolean).join(", "), selected: sel },
      layer: {
        insets: { end: lean + shaking, top: y },
        opacity: clamp(1 - Math.abs(off) * 0.14, 0.35, 1),
        layoutMotion: spring("ui"),
        motion: spring("ui"),
        states: [{ name: "pick", states: [vstate("open"), vstate("selected"), vstate("locked")], transitions: [] }],
        state: [{ group: "pick", state: s.locked ? "locked" : sel ? "selected" : "open" }],
      },
    }, () => {
      const rr = Math.max(6, t.panel);
      kit.setDraw({
        shape: { kind: "rect", radii: [rr, rr, rr, rr] },
        fills: [sel ? vgrad([[0, token("panelRaised", 0.98)], [1, token("panel", 0.98)]]) : { kind: "solid", color: token("panel", 0.82) }, vgrad([[0, token("sheen", sel ? 0.08 : 0.04)], [1, token("sheen", 0)]])],
        strokes: sel ? [line(u(2), token("focus"))] : ix.hovered || ix.focused ? [line(u(1.5), token("ink", 0.35))] : [line(1, token("ink", 0.08))],
        effects: [shadow(u(sel ? 8 : 3), u(sel ? 22 : 10), 0.5), ...(sel ? [glow(u(8), token("focus", 0.4), 0)] : [])],
      });
      const aj = h - u(16);
      artSlot("Jacket", "Songs.rowJacket", { texture: s.jacket ?? null, kind: "jacket", width: px(aj), height: px(aj), radius: Math.max(4, t.small) }, () => {
        if (s.locked) {
          kit.open("Dim", { width: grow(), height: grow() });
          kit.setDraw({ shape: { kind: "rect", radii: [Math.max(4, t.small), Math.max(4, t.small), Math.max(4, t.small), Math.max(4, t.small)] }, fills: [{ kind: "solid", color: token("shadow", 0.55) }] });
          kit.close();
        }
      });
      kit.column("Words", { width: grow(), gap: u(2) }, () => {
        text("Title", s.title ?? key, { step: st(sel ? 20 : 16), font: "display", nowrap: true, ellipsis: true, width: grow(), color: s.locked ? "inkMuted" : "ink" });
        text("Artist", s.artist ?? "", { step: st(13), color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
      });
      kit.column("End", { gap: u(4), cross: "end" }, () => {
        if (s.locked) symbol("Lock", "lock", u(16), role("inkMuted"), { icon: "lock" });
        else if (s.isNew) cornerTag("New", desc.newLabel ?? "New");
        if ((s.bpm ?? 0) > 0) kit.row("Bpm", { gap: u(4), cross: "end" }, () => {
          text("Label", desc.bpmLabel ?? "BPM", { step: "caption", font: "display", color: "inkMuted", nowrap: true });
          text("Value", String(Math.round(s.bpm!)), { step: st(sel ? 18 : 15), font: "mono", nowrap: true });
        });
      });
    });
    if (ix.focused && !sel && kit.memory(`~g.f.${id}`, 0) === 1) {
      out.selected = key;
      kit.feedback("hover", -1);
    }
    if (r.clicked) {
      if (!sel) {
        out.selected = key;
        kit.feedback("select", -1);
      } else if (s.locked) {
        out.refused = key;
        kit.feedback("error", -1);
      } else {
        out.play = true;
        kit.feedback("confirm", -1);
      }
    }
    // The selected row holds the focus when nothing else in the select has it (it opens on it, and keeps it as it moves).
    if (sel && kit.memory(`~g.f.${id}`, 0) === 0) kit.setFocus(kit.lastPath());
  }
  kit.close();
  rememberPath(`${id}~wheel`, kit.lastPath());
  kit.semantics({ role: "listbox", name: song?.title ?? "Songs" });
  kit.remember(`~g.f.${id}`, 1);
  if (portrait) previewBar(W);

  // Tab and the bumpers step the chart among the open ones; confirm on the wheel plays.
  const open = charts.filter((c) => !c.locked);
  const step = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
  if (step !== 0 && open.length > 1 && chart) {
    const at = Math.max(0, open.findIndex((c) => c.key === chart.key));
    out.difficulty = open[(at + step + open.length) % open.length]!.key ?? null;
    kit.feedback("tab", -1);
  }
  if (portrait) play(grow());
  if (out.refused) {
    kit.remember("~g.rt", tick);
    kit.remember("~g.rk", keyHash(out.refused));
  }
  kit.close();
  rememberRoom("g", id);
  return out;
}
