// og.ui.standingsTower, og.ui.lapTimer and og.ui.rearView: the racing rows
// of the genre sheets (B-racing-hud). Read against F1's and Gran Turismo's
// timing towers (a tabular position cell, the team colour as a thin
// stripe, the driver and the gap in tabular digits, the player's row lit,
// cars sliding to their new place), Forza's and F1's lap blocks (the place
// and the lap as large numerals over their totals, the running lap, the
// best and last laps, a signed split coloured ahead or behind) and the
// cockpit mirrors of Forza and Need for Speed (a render target flipped as a
// mirror in a dark rounded bezel with a glass highlight). The tower and the
// lap block share one width (raceWidth) so they stack as one column.

import type * as Api from "@origingame/engine";
import { BLACK, WHITE, a11y, absolute, anchorOf, apply, box, chip, drop, edge, frame, groups, hudPlace, hudRoom, keyId, kit, leaf, lit, paintRgba, pct, pop, px, role, sel, solid, sound, spring, stroke, teamColor, text, token, ts, u, ut, vgrad, stepSize, rtl, type Rgba, now, msToTicks, ticksToMs, reduced, clamp01, easeOut, pulse, glow, tsFit, claimAnnouncement } from "./core.ts";

/** The fastest lap's and a new best's purple, as F1 timing marks them. */
const PURPLE: Rgba = { r: 0.68, g: 0.42, b: 1, a: 1 };

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

/** The width the lap block and the tower share, so stacked they read as one column. */
function raceWidth(): number {
  const f = frame();
  return Math.min(hudRoom(), f.phone ? ut(220) : Math.max(u(220), Math.min(ut(290), hudRoom() * 0.3)));
}

/** The racing panels' ground: the HUD chip denser, over a blurred scene, so rows sit on an even ground. */
function racePanel(): Api.DrawDesc {
  const c = chip();
  return { ...c, fills: [solid(token("hud", 0.9)), ...(c.fills ?? []).slice(1)], effects: [{ kind: "backdrop", blur: u(12) }, ...(c.effects ?? [])] };
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
  const gap = 0;
  // At most two fifths of the safe area's height, so a large text size cuts the field rather than crowds the HUD
  // (it stacks under a lap block at the same corner and must leave the opposite corner free).
  const vp = f.viewport;
  const fit = Math.floor(((vp.height - vp.safeArea.top - vp.safeArea.bottom) * 0.4) / (rowH + gap));
  const most = Math.max(3, Math.min(desc.rows || (f.phone ? 5 : 10), fit));
  const pi = all.findIndex((e) => e.player);
  // A relative window: the player and the cars either side of it (at most one each side on a phone).
  const cap = (n: number | undefined) => Math.min(n ?? 0, f.phone ? 1 : 99);
  const windowed = pi >= 0 && cap(desc.ahead) + cap(desc.behind) > 0;
  // More cars than rows: the leaders, a break, then the cars either side of the player.
  let shown: Array<number | null> = all.map((_, i) => i);
  if (windowed) {
    const len = Math.min(all.length, cap(desc.ahead) + cap(desc.behind) + 1);
    const from = Math.max(0, Math.min(pi - cap(desc.ahead), all.length - len));
    shown = shown.slice(from, from + len);
  } else if (all.length > most && pi >= most - 1 && most < 5) {
    // Too few rows for a break: the cars either side of the player, the player kept in view.
    const from = Math.max(0, Math.min(pi - Math.floor((most - 1) / 2), all.length - most));
    shown = shown.slice(from, from + most);
  } else if (all.length > most && pi >= most - 1) {
    const lead = Math.max(1, most - 4);
    const around = [pi - 1, pi, pi + 1].filter((i) => i >= lead && i < all.length);
    shown = [...Array.from({ length: lead }, (_, i) => i), null, ...around];
  } else if (all.length > most) shown = shown.slice(0, most);
  const w = raceWidth();
  const posW = Math.ceil(stepSize(ts(14)).size * 1.4) + u(4);
  const gapW = Math.ceil(stepSize(ts(12)).size * 0.62 * 6) + u(4);
  const e = edge(`${id}~pos`, pi);
  sound("success", e.changed && pi >= 0 && e.prev >= 0 && pi < e.prev);
  const gapText = (en: Api.StandingsEntry, i: number): string => {
    if (en.out) return desc.outLabel ?? "OUT";
    if (en.pit) return desc.pitLabel ?? "PIT";
    if ((en.lapsDown ?? 0) > 0) return `+${en.lapsDown} L`;
    if (windowed) return en.player ? "" : signedSeconds(en.gap ?? 0);
    if (desc.relative) return en.player ? "\u2014" : signedSeconds(en.gap ?? 0);
    return i === 0 ? (desc.leaderLabel ?? "Leader") : signedSeconds(en.gap ?? 0);
  };
  hudPlace(id, anchorOf(desc, "left"), () => {
    box(id, { direction: "column", gap, width: px(w), padding: { left: 0, right: 0, top: u(3), bottom: u(3) }, part: sel("StandingsTower", "frame"), partClass: "hud" }, racePanel(), () => {
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
          // One mark for the player (docs/ui.md §15.5): a solid position cell over a lightly tinted row, no outline.
          fills: en.player ? [solid(windowed ? token("primary", 0.85) : token("highlight", 0.18))] : [],
        }, () => {
          // A hairline between rows, none against the player's lit row.
          const prev = r > 0 ? shown[r - 1] : null;
          if (r > 0 && !en.player && !(prev != null && all[prev]?.player)) leaf("Rule", { width: pct(1), height: px(1), part: sel("StandingsTower", "rule") }, { fills: [solid(lit(WHITE, 0.08))] }, absolute({ start: 0, top: 0 }));
          box("PositionBox", { width: px(posW), height: pct(1), align: "center", cross: "center", part: sel("StandingsTower", "position") }, en.player && !windowed ? {
            fills: [solid(token("highlight", 0.9))],
          } : undefined, () => {
            // In a window the player's place is the lap block's: its row shows none.
            if (!(windowed && en.player)) text("Position", String(i + 1), { step: ts(14), font: "display", color: en.player ? lit(BLACK) : token("ink"), nowrap: true });
          });
          leaf("Stripe", { width: px(Math.max(3, u(3))), height: pct(0.62), part: sel("StandingsTower", "stripe") }, { shape: { kind: "rect", radii: [1, 1, 1, 1] }, fills: [solid(lit(ink))] });
          text("Name", en.name ?? "", { step: ts(13), font: en.player ? "display" : "body", color: en.out ? token("inkFaint") : windowed && en.player ? token("onPrimary") : token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
          // Names win over marks: where the name would be cut, a rival's change arrow goes first, then the leader's
          // fastest-lap chip moves into the gap column in place of the leader label.
          const ch = en.change ?? 0;
          const fl = desc.fastestLabel ?? "FL";
          const flW = Math.ceil(fl.length * stepSize(ts(9)).size * 0.68) + u(8) + tk.sm;
          const chW = Math.ceil(stepSize(ts(10)).size * 1.4) + tk.sm;
          const nameW = (en.name ?? "").length * stepSize(ts(13)).size * 0.56;
          let room = w - posW - Math.max(3, u(3)) - tk.sm * 3 - gapW - tk.sm;
          let showCh = ch !== 0 && !en.out;
          let flIn: "name" | "gap" | "none" = en.fastest ? "name" : "none";
          if (flIn === "name") room -= flW;
          if (showCh) room -= chW;
          if (nameW > room && showCh && !en.player) { showCh = false; room += chW; }
          if (nameW > room && flIn === "name" && g === (desc.leaderLabel ?? "Leader")) { flIn = "gap"; room += flW; }
          const flChip = () => box("Fastest", { padding: { left: u(4), right: u(4), top: 0, bottom: 0 }, cross: "center", part: sel("StandingsTower", "fastest") }, {
            shape: { kind: "rect", radii: [u(3), u(3), u(3), u(3)] }, fills: [solid(lit(PURPLE))],
          }, () => text("Label", fl, { step: ts(9), font: "display", color: lit(WHITE), nowrap: true }), { a11y: a11y("image", "Fastest lap") });
          if (flIn === "name") flChip();
          if (showCh) {
            box("Change", { direction: "row", gap: 1, cross: "center", part: sel("StandingsTower", "change", ch > 0 ? "up" : "down") }, undefined, () => {
              const c = ch > 0 ? role("success") : role("danger");
              // Sized off the count's text, so it grows with the player's text size.
              arrow("Arrow", ch > 0, Math.max(u(7), Math.round(stepSize(ts(10)).size * 0.6)), c, sel("StandingsTower", "change"));
              text("Count", String(Math.abs(ch)), { step: ts(10), font: "mono", color: lit(c), nowrap: true });
            });
          }
          const gInk = en.pit ? lit(role("warn")) : en.out ? token("inkFaint") : windowed ? token("ink") : desc.relative && !en.player ? lit((en.gap ?? 0) < 0 ? role("danger") : role("success")) : token("ink");
          if (flIn === "gap") box("GapCell", { width: px(gapW), direction: "row", align: "end", cross: "center" }, undefined, flChip);
          else text("Gap", g, { step: ts(12), font: "mono", color: gInk, nowrap: true, width: px(gapW), align: "end" });
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
  const w = raceWidth();
  const lap = desc.lap ?? 0;
  const total = desc.lapTotal ?? 0;
  const final = total > 0 && lap >= total;
  const delta = desc.deltaMs ?? null;
  const split = delta === null ? "none" : desc.splitBest ? "best" : delta < 0 ? "ahead" : "behind";
  const show: Api.LapTimerShow = desc.show ?? "all";
  // No plate: every text outlined on the scene.
  const boxless = !!desc.boxless;
  const ol = boxless ? 1.2 : 0;
  const sectors = desc.sectors ?? [];
  const badge = !!desc.placeBadge;
  const ep = edge(`${id}~place`, desc.position ?? 0);
  const best = desc.bestMs ?? null;
  const last = desc.lastMs ?? null;
  const newBest = best !== null && last !== null && Math.abs(last - best) < 0.5;
  const es = edge(`${id}~split`, delta === null ? 1e9 : Math.round(delta));
  const eb = edge(`${id}~best`, best === null ? -1 : Math.round(best));
  sound("success", (es.changed && delta !== null && delta < 0 && es.prev >= 0) || (eb.changed && newBest && eb.prev >= 0));
  const big = (bid: string, caption: string, value: number, of: number, gold: boolean) => {
    box(bid, { direction: "column", gap: 0, part: sel("LapTimer", bid === "Lap" ? "lap" : "position", gold ? "final" : null) }, undefined, () => {
      text("Caption", caption, { step: ts(10), font: "display", color: gold ? lit(role("gold")) : token("inkMuted"), nowrap: true, outline: ol });
      // A number reads left to right in every locale: under a right-to-left layout, which mirrors the row, the total is
      // built first so it still lands after the value.
      // The badge's slant leans its top over the total: the row keeps that much room.
      const lean = badge && bid === "Position" ? Math.ceil(stepSize(ts(f.phone ? 48 : 64, true)).line * 0.18) : 0;
      kit.row("Num", { gap: u(2) + lean, cross: "end" }, () => {
        const total = () => { if (of > 0) text("Total", `/${of}`, { step: ts(f.phone ? 15 : 18), font: "display", color: token("inkMuted"), nowrap: true, outline: ol }); };
        if (rtl()) total();
        if (badge && bid === "Position") {
          // The arcade place: a huge ordinal that pops when it changes.
          const k = pop(ep.since, 0.3, 420);
          text("Value", desc.positionText || String(value), { step: ts(f.phone ? 48 : 64, true), font: "display", color: token("ink"), nowrap: true, outline: 2.4 },
            { transform: { scaleX: k, scaleY: k, skewX: -0.18, pivotX: 0, pivotY: 1 } });
        } else text("Value", String(value), { step: ts(f.phone ? 28 : 34, true), font: "display", color: gold ? lit(role("gold")) : token("ink"), nowrap: true, outline: ol });
        if (!rtl()) total();
      });
      // The lap's sectors as pips under it: driven ones in their timing colours, the rest gold on the final lap.
      if (bid === "Lap" && sectors.length > 0) {
        const pw = Math.max(u(8), Math.round(stepSize(ts(10)).size * 0.9));
        const ph = Math.max(3, u(4));
        kit.row("Sectors", { gap: Math.max(2, u(3)), part: sel("LapTimer", "sectors") }, () => {
          sectors.forEach((sct, k) => {
            const ink = sct === "best" ? lit(PURPLE) : sct === "personal" ? lit(role("success")) : sct === "slower" ? lit(role("warn")) : gold ? lit(role("gold"), 0.45) : token("inkFaint", 0.6);
            leaf(kit.idi("Sector", k), { width: px(pw), height: px(ph), part: sel("LapTimer", "sector", sct === "none" ? null : sct, gold ? "final" : null) }, {
              shape: { kind: "rect", radii: [1, 1, 1, 1] },
              fills: [solid(ink)],
              strokes: boxless ? [stroke(1, token("shadow", 0.8), "outside")] : [],
            });
          });
        });
      }
    });
  };
  const row = (rid: string, label: string, ms: number, ink: Api.Paint, scale = 1) => {
    box(rid, { direction: "row", gap: tk.sm, cross: "center", width: pct(1), part: sel("LapTimer", "row", rid.toLowerCase()) }, undefined, () => {
      text("Label", label, { step: ts(11), font: "display", color: token("inkMuted"), nowrap: true, ellipsis: true, width: { kind: "grow" }, outline: ol });
      text("Time", lapText(ms), { step: ts(14), font: "mono", color: ink, nowrap: true, outline: ol }, scale !== 1 ? { transform: { scaleX: scale, scaleY: scale, pivotX: 1 } } : undefined);
    });
  };
  // The split: a captioned row under the times, or a column in the head on a phone.
  const splitBox = (dir: "row" | "column") => {
    if (delta === null) return;
    const s = pop(es.since, 0.16, 320);
    const col = dir === "column";
    box("Split", { direction: dir, gap: col ? 0 : tk.sm, cross: col ? "end" : "center", width: col ? { kind: "grow" } : pct(1), part: sel("LapTimer", "split", split) }, undefined, () => {
      text("Label", desc.splitLabel ?? "Split", { step: ts(col ? 10 : 11), font: "display", color: token("inkMuted"), nowrap: true, ellipsis: col, outline: ol, ...(col ? {} : { width: { kind: "grow" as const } }) });
      text("Time", signedSeconds(delta / 1000), { step: ts(col ? 20 : 18), font: "mono", color: lit(desc.splitBest ? PURPLE : delta < 0 ? role("success") : role("danger")), nowrap: true, outline: ol }, s !== 1 ? { transform: { scaleX: s, scaleY: s, pivotX: 1 } } : undefined);
    });
  };
  const headed = show !== "times" && ((desc.position ?? 0) > 0 || lap > 0);
  const timed = show !== "head";
  hudPlace(id, anchorOf(desc, "topLeft"), () => {
    box(id, {
      direction: "column", gap: u(4), width: px(w), padding: { left: tk.md, right: tk.md, top: tk.sm, bottom: tk.sm },
      part: sel("LapTimer", "frame"), partClass: "hud",
    }, boxless ? undefined : racePanel(), () => {
      if (headed) {
        kit.row("Head", { gap: tk.md, cross: "end", width: pct(1) }, () => {
          if ((desc.position ?? 0) > 0) big("Position", desc.positionLabel ?? "Position", desc.position ?? 0, desc.positionTotal ?? 0, false);
          if ((desc.position ?? 0) > 0 && lap > 0 && !boxless) leaf("Divider", { width: px(1), height: px(ut(38)) }, { fills: [solid(token("line", 0.9))] });
          if (lap > 0) big("Lap", final ? (desc.finalLabel ?? "Final lap") : (desc.lapLabel ?? "Lap"), lap, total, final);
          // A phone has no height for a second line: the split joins the head as a third column.
          if (f.phone && delta !== null && show === "all") {
            if (!boxless) leaf("Divider2", { width: px(1), height: px(ut(38)) }, { fills: [solid(token("line", 0.9))] });
            splitBox("column");
          }
        });
        if ((!f.phone || delta === null) && timed && !boxless) leaf("Rule", { width: pct(1), height: px(1) }, { fills: [solid(token("line", 0.7))] });
      }
      if (!f.phone && timed) {
        row("Current", desc.currentLabel ?? "Current lap", desc.currentMs ?? 0, token("ink"));
        if (best !== null) row("Best", desc.bestLabel ?? "Best lap", best, newBest && eb.since < 240 ? lit(PURPLE) : token("ink"));
        if (last !== null) row("Last", desc.lastLabel ?? "Last lap", last, newBest ? lit(PURPLE) : token("inkMuted"));
      }
      if (delta !== null && timed && !(f.phone && headed && show === "all")) splitBox("row");
    }, {
      a11y: a11y("status", `Position ${desc.position ?? 0} of ${desc.positionTotal ?? 0}, lap ${lap}${total ? ` of ${total}` : ""}`, { value: [lapText(desc.currentMs ?? 0), best !== null ? `best ${lapText(best)}` : "", delta !== null ? `split ${signedSeconds(delta / 1000)}` : ""].filter(Boolean).join(", ") }),
      ...groups({ lap: final ? "final" : "racing", split, best: newBest ? "new" : "none" }, { lap: ["racing", "final"], split: ["none", "ahead", "behind", "best"], best: ["none", "new"] }),
    });
  });
}

export function rearView(id: string, desc: Api.RearViewDesc = {}): void {
  const f = frame();
  const room = hudRoom();
  const w = Math.round(Math.min(room, desc.width ? u(desc.width) : Math.min(u(560), room * (f.viewport.breakpoint === "phonePortrait" ? 2 / 3 : f.phone ? 0.4 : 1 / 3))));
  const h = Math.round(w / Math.max(1.5, desc.aspect || 4.2));
  // A dark bezel round the glass, as a cockpit mirror's housing: square-on, its corners rounded, a soft shadow under it.
  const bezel = Math.max(5, u(7));
  const r = Math.round(Math.min(h * 0.22, u(18)));
  const ri = Math.max(2, r - bezel);
  const gw = w - bezel * 2;
  const gh = h - bezel * 2;
  hudPlace(id, anchorOf(desc, "top"), () => {
    box(id, { width: px(w), height: px(h), padding: { left: bezel, right: bezel, top: bezel, bottom: bezel }, part: sel("RearView", "frame"), partClass: "hud" }, {
      shape: { kind: "rect", radii: [r, r, r, r] },
      fills: [solid(lit({ r: 0.043, g: 0.05, b: 0.063, a: 1 }, 0.96)), vgrad([[0, lit(WHITE, 0.07)], [0.5, lit(WHITE, 0)]])],
      strokes: [stroke(1, lit(WHITE, 0.08)), stroke(1, lit(BLACK, 0.6), "outside")],
      effects: [drop(u(4), u(16), token("shadow", 0.55))],
    }, () => {
      box("Glass", { width: px(gw), height: px(gh), part: sel("RearView", "glass") }, {
        shape: { kind: "rect", radii: [ri, ri, ri, ri] },
        fills: [solid(lit({ r: 0.1, g: 0.12, b: 0.15, a: 1 }))],
        clip: "shape",
      }, () => {
        if (desc.texture) {
          kit.image("View", { texture: desc.texture, width: px(gw), height: px(gh), label: "" });
          apply(absolute({ start: 0, top: 0 }, desc.mirror !== false ? { transform: { scaleX: -1 } } : {}));
        }
        // The glass: a faint sheen across its top and a bright hairline along its top edge.
        leaf("Glare", { width: pct(1), height: px(Math.round(gh * 0.5)), part: sel("RearView", "glare") }, { fills: [vgrad([[0, lit(WHITE, 0.14)], [1, lit(WHITE, 0)]])] }, absolute({ start: 0, top: 0 }));
        leaf("Edge", { width: pct(1), height: px(1), part: sel("RearView", "glare") }, { fills: [solid(lit(WHITE, 0.32))] }, absolute({ start: 0, top: 0 }));
      });
    }, { a11y: a11y("image", desc.label ?? "Rear view") });
  });
}

const BANNER_MS: Record<Exclude<Api.RaceEvent, "none" | "countdown">, number> = { go: 1000, checkpoint: 1500, finalLap: 2000, finish: 2500 };
/** A countdown numeral's scale-in and every banner's fade, ms. */
const IN_MS = 280;
const FADE_MS = 200;

/**
 * og.ui.raceBanner: the racing HUD's centre-third moments, one at a time,
 * by the tick from `at` (a still at a tick is always the same). Wrong way
 * wins over the others while it holds.
 */
export function raceBanner(id: string, desc: Api.RaceBannerDesc = {}): void {
  const f = frame();
  const vp = f.viewport;
  const t = now();
  const at = desc.at ?? 0;
  const age = t - at;
  const still = reduced();
  const count = Math.max(1, Math.min(9, desc.count || 3));
  const sec = msToTicks(1000);
  // What shows this tick.
  let event: Api.RaceEvent | "wrongWay" = desc.event ?? "none";
  if (desc.wrongWay) event = "wrongWay";
  else if (age < 0) event = "none";
  else if (event === "countdown" && age >= count * sec + sec) event = "none";
  else if (event !== "none" && event !== "countdown" && age >= msToTicks(BANNER_MS[event])) event = "none";
  const g = groups({ event }, { event: ["none", "countdown", "go", "checkpoint", "finalLap", "finish", "wrongWay"] });
  // Fades in and out over its life (none under reduced motion).
  const life = (ms: number): number => (still ? 1 : Math.min(clamp01(ticksToMs(age) / FADE_MS), clamp01((ms - ticksToMs(age)) / FADE_MS)));
  const short = Math.min(vp.width, vp.height);
  if (event === "none") return;
  claimAnnouncement();

  if (event === "countdown" || event === "go") {
    // The start: the numeral of this second over the lights, then GO.
    const n = event === "go" ? 0 : count - Math.floor(age / sec);
    const go = n <= 0;
    const inSec = event === "go" ? age : age % sec;
    const k = still ? 1 : 1 + 0.4 * (1 - easeOut(ticksToMs(inSec) / IN_MS));
    const fade = still ? 1 : Math.min(clamp01(ticksToMs(inSec) / 80), go ? clamp01((1000 - ticksToMs(inSec)) / FADE_MS) : 1);
    const lightsOn = go ? count : count - n + 1;
    const d = Math.max(u(18), Math.round(short * 0.045));
    const step = tsFit(120, short * 0.3, true);
    const label = go ? (desc.goLabel ?? "GO!") : String(n);
    kit.anchor(id, { anchor: "center", offsetY: -Math.round(vp.height * 0.12), direction: "column", cross: "center", gap: u(10), over: true }, () => {
      box("Lights", { direction: "row", gap: Math.round(d * 0.6), padding: { left: u(8), right: u(8), top: u(6), bottom: u(6) }, part: sel("RaceBanner", "lights") }, chip(d), () => {
        for (let i = 0; i < count; i++) {
          const on = i < lightsOn;
          const hue = go ? role("success") : role("danger");
          leaf(kit.idi("Light", i), { width: px(d), height: px(d), part: sel("RaceBanner", "light", on ? (go ? "go" : "on") : null) }, {
            shape: { kind: "ellipse" },
            fills: [solid(on ? lit(hue) : token("shadow", 0.7))],
            strokes: [stroke(1, token("lineStrong", 0.6))],
            effects: on ? [glow(d * 0.6, lit(hue, 0.6))] : [],
          });
        }
      });
      text("Count", label, { step, font: "display", color: go ? lit(role("success")) : token("ink"), nowrap: true, outline: 2.4 },
        { transform: { scaleX: k, scaleY: k }, opacity: fade });
    });
    apply({ ...g, a11y: a11y("status", label), inert: true });
    return;
  }

  if (event === "checkpoint") {
    const delta = desc.deltaMs ?? 0;
    const ahead = delta < 0;
    const ink = desc.best ? PURPLE : ahead ? role("success") : role("danger");
    const label = (desc.checkpointLabel ?? "Checkpoint {n}").replaceAll("{n}", String(desc.index || 1));
    const value = signedSeconds(delta / 1000);
    // Centre-top, under the lap time (the top band's first row).
    kit.anchor(id, { anchor: "top", offsetY: Math.round(vp.height * 0.11), over: true }, () => {
      box("Split", { direction: "column", cross: "center", gap: 0, padding: { left: u(14), right: u(14), top: u(4), bottom: u(6) }, part: sel("RaceBanner", "checkpoint", desc.best ? "best" : ahead ? "ahead" : "behind") }, chip(), () => {
        text("Label", label, { step: ts(12), color: token("inkMuted"), nowrap: true });
        text("Delta", value, { step: ts(30, true), font: "mono", color: lit(ink), nowrap: true, outline: 1.2 });
      }, { opacity: life(BANNER_MS.checkpoint), transform: { scaleX: pop(age, 0.12), scaleY: pop(age, 0.12) } });
    });
    apply({ ...g, a11y: a11y("status", `${label} ${value}`), inert: true });
    return;
  }

  if (event === "finalLap") {
    // A thin gold band sweeping in across the top third, its words on it.
    const W = vp.width;
    const x = still ? 0 : -W * (1 - easeOut(ticksToMs(age) / 450));
    const label = desc.finalLapLabel ?? "Final lap";
    kit.anchor(id, { anchor: "top", width: pct(1), margin: 0, offsetY: Math.round(vp.height * 0.22), over: true }, () => {
      box("Band", { width: pct(1), height: px(Math.max(u(34), stepSize(ts(20, true)).line + u(8))), align: "center", cross: "center", part: sel("RaceBanner", "band", "final") }, {
        fills: [vgrad([[0, token("gold", 0)], [0.2, token("gold", 0.85)], [0.8, token("gold", 0.85)], [1, token("gold", 0)]], 90)],
      }, () => {
        text("Label", label.toUpperCase(), { step: ts(20, true), font: "display", color: lit(BLACK), nowrap: true });
      }, { opacity: life(BANNER_MS.finalLap), transform: { x } });
    });
    apply({ ...g, a11y: a11y("status", label), inert: true });
    return;
  }

  if (event === "finish") {
    // A checker wipe across the middle, then the place huge with the time under it.
    const W = vp.width;
    const h = Math.max(u(40), Math.round(vp.height * 0.07));
    const cell = Math.round(h / 2);
    const wipe = still ? 1 : easeOut(ticksToMs(age) / 600);
    const placeText = desc.placeText || String(desc.place || 1);
    const label = desc.finishLabel ?? "Finish";
    const time = lapText(desc.timeMs ?? 0);
    const reveal = still ? 1 : clamp01((ticksToMs(age) - 350) / 200);
    const k = pop(age - msToTicks(350), 0.2);
    kit.anchor(id, { anchor: "center", width: pct(1), margin: 0, direction: "column", cross: "center", gap: u(8), over: true }, () => {
      box("Checker", { direction: "row", width: pct(1), height: px(h), part: sel("RaceBanner", "checker") }, undefined, () => {
        const cols = Math.ceil(W / cell);
        for (let i = 0; i < cols; i++) {
          const a = i % 2 === 0;
          const [top, bottom] = a ? [WHITE, BLACK] : [BLACK, WHITE];
          leaf(kit.idi("Col", i), { width: px(cell), height: px(h) }, {
            fills: [vgrad([[0, lit(top, 0.9)], [0.5, lit(top, 0.9)], [0.5, lit(bottom, 0.9)], [1, lit(bottom, 0.9)]])],
          });
        }
      }, { transform: { x: -W * (1 - wipe) } });
      box("Result", { direction: "column", cross: "center", gap: 0 }, undefined, () => {
        text("Label", label, { step: ts(16), color: token("inkMuted"), nowrap: true, outline: 1 });
        text("Place", placeText, { step: tsFit(120, short * 0.26, true), font: "display", color: token("ink"), nowrap: true, outline: 2.4 });
        text("Time", time, { step: ts(22), font: "mono", color: token("ink"), nowrap: true, outline: 1.2 });
      }, { opacity: reveal * life(BANNER_MS.finish), transform: { scaleX: k, scaleY: k } });
    });
    apply({ ...g, a11y: a11y("status", `${label} ${placeText} ${time}`), inert: true });
    return;
  }

  // Wrong way: danger, a U-turn arrow, pulsing once a second.
  const label = desc.wrongWayLabel ?? "Wrong way";
  const s = Math.max(u(28), Math.round(short * 0.06), Math.round(stepSize(ts(26, true)).line * 0.75));
  const beat = still ? 1 : 0.55 + 0.45 * pulse(1);
  kit.anchor(id, { anchor: "center", offsetY: -Math.round(vp.height * 0.12), over: true }, () => {
    box("WrongWay", { direction: "row", gap: u(10), cross: "center", padding: { left: u(16), right: u(18), top: u(8), bottom: u(8) }, part: sel("RaceBanner", "wrongWay") }, {
      ...chip(),
      fills: [solid(lit(role("danger"), 0.88))],
      effects: [glow(16, lit(role("danger"), 0.5))],
    }, () => {
      box("Arrow", { width: px(s), height: px(s) }, undefined, () => {
        leaf("Turn", { width: px(s), height: px(s) }, {
          shape: { kind: "path", points: [{ x: 0.72 * s, y: 0.92 * s }, { x: 0.72 * s, y: 0.4 * s }, { x: 0.64 * s, y: 0.2 * s }, { x: 0.5 * s, y: 0.13 * s }, { x: 0.36 * s, y: 0.2 * s }, { x: 0.28 * s, y: 0.4 * s }, { x: 0.28 * s, y: 0.62 * s }], width: Math.max(3, s * 0.12), cap: "round", join: "round" },
          fills: [solid(lit(WHITE))],
        }, absolute({ start: 0, top: 0 }));
        leaf("Head", { width: px(s), height: px(s) }, {
          shape: { kind: "path", points: [{ x: 0.1 * s, y: 0.6 * s }, { x: 0.46 * s, y: 0.6 * s }, { x: 0.28 * s, y: 0.92 * s }], closed: true, filled: true, join: "round" },
          fills: [solid(lit(WHITE))],
        }, absolute({ start: 0, top: 0 }));
      });
      text("Label", label.toUpperCase(), { step: ts(26, true), font: "display", color: lit(WHITE), nowrap: true });
    }, { opacity: beat });
  });
  apply({ ...g, a11y: a11y("status", label), inert: true });
}
