// og.ui.standingsTower, og.ui.lapTimer and og.ui.rearView: the racing rows
// of the genre sheets (B-racing-hud). Read against F1's and Gran Turismo's
// timing towers (a tabular position cell, the team colour as a thin
// stripe, the driver and the gap in tabular digits, the player's row lit,
// cars sliding to their new place), Forza's and F1's lap blocks (the place
// and the lap as large numerals over their totals, the running lap, the
// best and last laps, a signed split coloured ahead or behind) and the
// cockpit mirrors of Forza and Need for Speed (a render target flipped as a
// mirror in a wide frame whose sides slant in).

import type * as Api from "@origingame/engine";
import { BLACK, WHITE, a11y, absolute, anchorOf, apply, box, chip, drop, edge, frame, groups, hudPlace, hudRoom, keyId, kit, leaf, lit, paintRgba, pct, pop, px, role, sel, solid, sound, spring, stroke, teamColor, text, token, ts, u, ut, vgrad, stepSize, rtl, type Rgba } from "./core.ts";

/** A lap time as racing games write it: m:ss.mmm. */
export function lapText(ms: number): string {
  const t = Math.max(0, Math.round(ms));
  const m = Math.floor(t / 60000);
  const s = Math.floor((t % 60000) / 1000);
  return `${m}:${String(s).padStart(2, "0")}.${String(t % 1000).padStart(3, "0")}`;
}

/** A signed gap in seconds, with a true minus: −0.68, +1.17. */
export function signedSeconds(sec: number, digits = 2): string {
  const v = Math.abs(sec).toFixed(digits);
  if (Number(v) === 0) return (0).toFixed(digits);
  return sec < 0 ? `\u2212${v}` : `+${v}`;
}

/** A small filled triangle pointing up or down. */
function arrow(id: string, up: boolean, side: number, ink: Rgba, part: string): void {
  const pts = (up ? [[0.5, 0.1], [0.95, 0.9], [0.05, 0.9]] : [[0.05, 0.1], [0.95, 0.1], [0.5, 0.9]]).map(([x, y]) => ({ x: x! * side, y: y! * side }));
  leaf(id, { width: px(side), height: px(side), part }, { shape: { kind: "path", points: pts, closed: true, filled: true, join: "round" }, fills: [solid(lit(ink))] });
}

export function standingsTower(id: string, desc: Api.StandingsTowerDesc = {}): void {
  const f = frame();
  const tk = f.t;
  const all = desc.entries ?? [];
  const rowH = Math.max(ut(24), stepSize(ts(13)).line + u(6));
  const gap = u(2);
  // At most two fifths of the safe area's height, so a large text size cuts the field rather than crowds the HUD
  // (it stacks under a lap block at the same corner and must leave the opposite corner free).
  const vp = f.viewport;
  const fit = Math.floor(((vp.height - vp.safeArea.top - vp.safeArea.bottom) * 0.4) / (rowH + gap));
  const most = Math.max(3, Math.min(desc.rows || (f.phone ? 5 : 10), fit));
  const pi = all.findIndex((e) => e.player);
  // More cars than rows: the leaders, a break, then the cars either side of the player.
  let shown: Array<number | null> = all.map((_, i) => i);
  if (all.length > most && pi >= most - 1 && most < 5) {
    // Too few rows for a break: the cars either side of the player, the player kept in view.
    const from = Math.max(0, Math.min(pi - Math.floor((most - 1) / 2), all.length - most));
    shown = shown.slice(from, from + most);
  } else if (all.length > most && pi >= most - 1) {
    const lead = Math.max(1, most - 4);
    const around = [pi - 1, pi, pi + 1].filter((i) => i >= lead && i < all.length);
    shown = [...Array.from({ length: lead }, (_, i) => i), null, ...around];
  } else if (all.length > most) shown = shown.slice(0, most);
  const w = Math.min(hudRoom(), ut(f.phone ? 220 : 270));
  const posW = Math.ceil(stepSize(ts(14)).size * 1.4) + u(4);
  const gapW = Math.ceil(stepSize(ts(12)).size * 0.62 * 6) + u(4);
  const e = edge(`${id}~pos`, pi);
  sound("success", e.changed && pi >= 0 && e.prev >= 0 && pi < e.prev);
  const gapText = (en: Api.StandingsEntry, i: number): string => {
    if (en.out) return desc.outLabel ?? "OUT";
    if (en.pit) return desc.pitLabel ?? "PIT";
    if ((en.lapsDown ?? 0) > 0) return `+${en.lapsDown} L`;
    if (desc.relative) return en.player ? "\u2014" : signedSeconds(en.gap ?? 0);
    return i === 0 ? (desc.leaderLabel ?? "Leader") : signedSeconds(en.gap ?? 0);
  };
  hudPlace(id, anchorOf(desc, "left"), () => {
    box(id, { direction: "column", gap, width: px(w), padding: { left: u(4), right: u(4), top: u(4), bottom: u(4) }, part: sel("StandingsTower", "frame"), partClass: "hud" }, chip(), () => {
      shown.forEach((i, r) => {
        if (i === null) {
          // The break: three dots where the field is cut.
          box("Break", { direction: "row", gap: u(3), align: "center", cross: "center", width: pct(1), height: px(Math.round(rowH * 0.5)), part: sel("StandingsTower", "break") }, undefined, () => {
            for (let d = 0; d < 3; d++) leaf(kit.idi("Dot", d), { width: px(u(3)), height: px(u(3)) }, { shape: { kind: "ellipse" }, fills: [solid(token("inkMuted"))] });
          });
          return;
        }
        const en = all[i]!;
        const ink = paintRgba(en.color, teamColor(i));
        const state = en.out ? "out" : en.pit ? "pit" : en.player ? "player" : "racing";
        // A car that changes place slides there from its old row.
        const y = r * (rowH + gap);
        const slide = spring(`${id}/${en.key ?? i}~y`, y, "ui") - y;
        const g = gapText(en, i);
        box(keyId(en.key ?? en.name ?? String(i)), {
          direction: "row", gap: tk.sm, cross: "center", width: pct(1), height: px(rowH), padding: { left: 0, right: tk.sm, top: 0, bottom: 0 },
          part: sel("StandingsTower", "row", en.player ? "player" : null, en.pit ? "pit" : null, en.out ? "out" : null),
        }, {
          shape: { kind: "rect", radii: [3, 3, 3, 3] },
          fills: [solid(en.player ? token("highlight", 0.24) : r % 2 === 0 ? token("shadow", 0.3) : token("shadow", 0.16))],
          strokes: en.player ? [stroke(1, token("highlight", 0.85))] : [],
        }, () => {
          box("PositionBox", { width: px(posW), height: pct(1), align: "center", cross: "center", part: sel("StandingsTower", "position") }, {
            shape: { kind: "rect", radii: [3, 0, 0, 3] },
            fills: [solid(en.player ? token("highlight", 0.9) : lit(BLACK, 0.35))],
          }, () => text("Position", String(i + 1), { step: ts(14), font: "display", color: en.player ? lit(BLACK) : token("ink"), nowrap: true }));
          leaf("Stripe", { width: px(Math.max(3, u(3))), height: pct(0.62), part: sel("StandingsTower", "stripe") }, { shape: { kind: "rect", radii: [1, 1, 1, 1] }, fills: [solid(lit(ink))] });
          text("Name", en.name ?? "", { step: ts(13), font: en.player ? "display" : "body", color: en.out ? token("inkFaint") : token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
          if (en.fastest) leaf("Fastest", { width: px(u(6)), height: px(u(6)), part: sel("StandingsTower", "fastest") }, { shape: { kind: "ellipse" }, fills: [solid(lit({ r: 0.66, g: 0.36, b: 0.98, a: 1 }))] }, { a11y: a11y("image", "Fastest lap") });
          const ch = en.change ?? 0;
          if (ch !== 0 && !en.out) {
            box("Change", { direction: "row", gap: 1, cross: "center", part: sel("StandingsTower", "change", ch > 0 ? "up" : "down") }, undefined, () => {
              const c = ch > 0 ? role("success") : role("danger");
              arrow("Arrow", ch > 0, u(7), c, sel("StandingsTower", "change"));
              text("Count", String(Math.abs(ch)), { step: ts(10), font: "mono", color: lit(c), nowrap: true });
            });
          }
          const gInk = en.pit ? lit(role("warn")) : en.out ? token("inkFaint") : desc.relative && !en.player ? lit((en.gap ?? 0) < 0 ? role("danger") : role("success")) : token("ink");
          text("Gap", g, { step: ts(12), font: "mono", color: gInk, nowrap: true, width: px(gapW), align: "end" });
        }, {
          opacity: en.out ? 0.6 : 1,
          ...(slide !== 0 ? { transform: { y: slide } } : {}),
          a11y: a11y("row", en.name ?? "", { value: `P${i + 1}, ${g}`, selected: !!en.player }),
          ...groups({ row: state }, { row: ["racing", "player", "pit", "out"] }),
        });
      });
    }, { a11y: a11y("table", "Standings") });
  });
}

export function lapTimer(id: string, desc: Api.LapTimerDesc = {}): void {
  const f = frame();
  const tk = f.t;
  // Grows with the text, to three tenths of the HUD band off a phone so a centred mirror keeps its place beside it;
  // on a phone, wide enough for the place, the lap and the split in one head.
  const w = Math.min(hudRoom(), f.phone ? ut(210) : Math.max(u(200), Math.min(ut(290), hudRoom() * 0.3)));
  const lap = desc.lap ?? 0;
  const total = desc.lapTotal ?? 0;
  const final = total > 0 && lap >= total;
  const delta = desc.deltaMs ?? null;
  const split = delta === null ? "none" : delta < 0 ? "ahead" : "behind";
  const best = desc.bestMs ?? null;
  const last = desc.lastMs ?? null;
  const newBest = best !== null && last !== null && Math.abs(last - best) < 0.5;
  const es = edge(`${id}~split`, delta === null ? 1e9 : Math.round(delta));
  const eb = edge(`${id}~best`, best === null ? -1 : Math.round(best));
  sound("success", (es.changed && delta !== null && delta < 0 && es.prev >= 0) || (eb.changed && newBest && eb.prev >= 0));
  const big = (bid: string, caption: string, value: number, of: number, gold: boolean) => {
    box(bid, { direction: "column", gap: 0, part: sel("LapTimer", bid === "Lap" ? "lap" : "position", gold ? "final" : null) }, undefined, () => {
      text("Caption", caption, { step: ts(10), font: "display", color: gold ? lit(role("gold")) : token("inkMuted"), nowrap: true });
      // A number reads left to right in every locale: under a right-to-left layout, which mirrors the row, the total is
      // built first so it still lands after the value.
      kit.row("Num", { gap: u(2), cross: "end" }, () => {
        const total = () => { if (of > 0) text("Total", `/${of}`, { step: ts(f.phone ? 15 : 18), font: "display", color: token("inkMuted"), nowrap: true }); };
        if (rtl()) total();
        text("Value", String(value), { step: ts(f.phone ? 28 : 34, true), font: "display", color: gold ? lit(role("gold")) : token("ink"), nowrap: true });
        if (!rtl()) total();
      });
    });
  };
  const row = (rid: string, label: string, ms: number, ink: Api.Paint, scale = 1) => {
    box(rid, { direction: "row", gap: tk.sm, cross: "center", width: pct(1), part: sel("LapTimer", "row", rid.toLowerCase()) }, undefined, () => {
      text("Label", label, { step: ts(11), font: "display", color: token("inkMuted"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
      text("Time", lapText(ms), { step: ts(14), font: "mono", color: ink, nowrap: true }, scale !== 1 ? { transform: { scaleX: scale, scaleY: scale, pivotX: 1 } } : undefined);
    });
  };
  // The split: a captioned row under the times, or a column in the head on a phone.
  const splitBox = (dir: "row" | "column") => {
    if (delta === null) return;
    const s = pop(es.since, 0.16, 320);
    const col = dir === "column";
    box("Split", { direction: dir, gap: col ? 0 : tk.sm, cross: col ? "end" : "center", width: col ? { kind: "grow" } : pct(1), part: sel("LapTimer", "split", split) }, undefined, () => {
      text("Label", desc.splitLabel ?? "Split", { step: ts(col ? 10 : 11), font: "display", color: token("inkMuted"), nowrap: true, ellipsis: col, ...(col ? {} : { width: { kind: "grow" as const } }) });
      text("Time", signedSeconds(delta / 1000), { step: ts(col ? 20 : 18), font: "mono", color: lit(delta < 0 ? role("success") : role("danger")), nowrap: true }, s !== 1 ? { transform: { scaleX: s, scaleY: s, pivotX: 1 } } : undefined);
    });
  };
  const headed = (desc.position ?? 0) > 0 || lap > 0;
  hudPlace(id, anchorOf(desc, "topLeft"), () => {
    box(id, {
      direction: "column", gap: u(4), width: px(w), padding: { left: tk.md, right: tk.md, top: tk.sm, bottom: tk.sm },
      part: sel("LapTimer", "frame"), partClass: "hud",
    }, chip(), () => {
      if (headed) {
        kit.row("Head", { gap: tk.md, cross: "end", width: pct(1) }, () => {
          if ((desc.position ?? 0) > 0) big("Position", desc.positionLabel ?? "Position", desc.position ?? 0, desc.positionTotal ?? 0, false);
          if ((desc.position ?? 0) > 0 && lap > 0) leaf("Divider", { width: px(1), height: px(ut(38)) }, { fills: [solid(token("line", 0.9))] });
          if (lap > 0) big("Lap", final ? (desc.finalLabel ?? "Final lap") : (desc.lapLabel ?? "Lap"), lap, total, final);
          // A phone has no height for a second line: the split joins the head as a third column.
          if (f.phone && delta !== null) {
            leaf("Divider2", { width: px(1), height: px(ut(38)) }, { fills: [solid(token("line", 0.9))] });
            splitBox("column");
          }
        });
        if (!f.phone || delta === null) leaf("Rule", { width: pct(1), height: px(1) }, { fills: [solid(token("line", 0.7))] });
      }
      if (!f.phone) {
        row("Current", desc.currentLabel ?? "Current lap", desc.currentMs ?? 0, token("ink"));
        if (best !== null) row("Best", desc.bestLabel ?? "Best lap", best, newBest && eb.since < 240 ? lit({ r: 0.72, g: 0.45, b: 1, a: 1 }) : token("ink"));
        if (last !== null) row("Last", desc.lastLabel ?? "Last lap", last, newBest ? lit({ r: 0.72, g: 0.45, b: 1, a: 1 }) : token("inkMuted"));
      }
      if (delta !== null && !(f.phone && headed)) splitBox("row");
    }, {
      a11y: a11y("status", `Position ${desc.position ?? 0} of ${desc.positionTotal ?? 0}, lap ${lap}${total ? ` of ${total}` : ""}`, { value: [lapText(desc.currentMs ?? 0), best !== null ? `best ${lapText(best)}` : "", delta !== null ? `split ${signedSeconds(delta / 1000)}` : ""].filter(Boolean).join(", ") }),
      ...groups({ lap: final ? "final" : "racing", split, best: newBest ? "new" : "none" }, { lap: ["racing", "final"], split: ["none", "ahead", "behind"], best: ["none", "new"] }),
    });
  });
}

export function rearView(id: string, desc: Api.RearViewDesc = {}): void {
  const f = frame();
  const room = hudRoom();
  const w = Math.round(Math.min(room, desc.width ? u(desc.width) : Math.min(u(560), room * (f.viewport.breakpoint === "phonePortrait" ? 2 / 3 : f.phone ? 0.4 : 1 / 3))));
  const h = Math.round(w / Math.max(1.5, desc.aspect || 4.2));
  const slant = Math.round(h * 0.16);
  const rim = Math.max(2, u(3));
  const pts = [{ x: 0, y: 0 }, { x: w, y: 0 }, { x: w - slant, y: h }, { x: slant, y: h }];
  hudPlace(id, anchorOf(desc, "top"), () => {
    box(id, { width: px(w), height: px(h), part: sel("RearView", "frame"), partClass: "hud" }, {
      shape: { kind: "path", points: pts, closed: true, filled: true, join: "round" },
      fills: [solid(lit(BLACK, 0.85))],
      strokes: [stroke(rim, token("lineStrong", 0.85)), stroke(1, token("shadow", 0.9), "outside")],
      effects: [drop(u(2), u(10), token("shadow", 0.5))],
      clip: "shape",
    }, () => {
      if (desc.texture) {
        kit.image("View", { texture: desc.texture, width: px(w), height: px(h), label: "" });
        apply(absolute({ start: 0, top: 0 }, desc.mirror !== false ? { transform: { scaleX: -1 } } : {}));
      }
      // A soft glare across the glass's top.
      leaf("Glare", { width: pct(1), height: px(Math.round(h * 0.45)), part: sel("RearView", "glare") }, { fills: [vgrad([[0, lit(WHITE, 0.16)], [1, lit(WHITE, 0)]])] }, absolute({ start: 0, top: 0 }));
    }, { a11y: a11y("image", desc.label ?? "Rear view") });
  });
}
