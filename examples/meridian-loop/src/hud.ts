// The race's HUD as the game shipped it (the engine's racing HUD of
// November 2026, a time trial's: no place, no boost), now the game's own
// code since U8 took the fixed templates away: the lap top left, the
// running lap time and Pause top right over the record, the sectors and
// the minimap, the speed and gear on a dash bottom right with its rev
// meter, and the touch controls on a phone. Every box is on the style's
// HUD chip, so its words never sit bare on the road.

import { og } from "@origingame/engine";
import type { Insets, MapMarker, Paint, Texture } from "@origingame/engine";

export interface HudSplit {
  label: string;
  /** Seconds once the sector is crossed; null not yet. */
  time: number | null;
  /** Against the record's sector; null without one. */
  delta: number | null;
}

export interface HudDesc {
  speed: number;
  maxSpeed: number;
  gear: string;
  lap: number;
  laps: number;
  /** The lap's running time, seconds. */
  time: number;
  /** The last split's (or the line's) delta, seconds; null hides it. */
  delta: number | null;
  /** The lap record, seconds; null: none. */
  best: number | null;
  splits: HudSplit[];
  map: { texture: Texture; markers: MapMarker[]; size: number };
  touch: boolean;
  unit: string;
  lapLabel: string;
  /** The record's line, `{best}` its time. */
  bestLabel: string;
  pauseLabel: string;
  brakeLabel: string;
  leftLabel: string;
  rightLabel: string;
}

export interface HudOut {
  paused: boolean;
  moveX: number;
  held: string[];
}

const token = (name: string): Paint => ({ token: name }) as Paint;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;
const grow = { kind: "grow" } as const;
const xy = (x: number, y: number): Insets => ({ left: x, top: y, right: x, bottom: y });

/** The frame's tokens: the style's spaces, radii and control sizes. */
function tokens() {
  const theme = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const s = (k: string): number => theme.space[k] ?? 0;
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  return {
    xxs: s("xxs"), xs: s("xs"), sm: s("sm"), md: s("md"), lg: s("lg"),
    control: theme.radius["control"] ?? 0,
    controlMd: theme.sizes.controlMd,
    phone,
    touch: vp.coarsePointer || phone || vp.breakpoint === "tablet",
  };
}

/** A finished time as m:ss.mmm, cut to the millisecond (never a millisecond short by a float's last bit). */
function lapTime(seconds: number): string {
  const ms = Math.floor(Math.max(0, seconds) * 1000 + 1e-6);
  const m = Math.floor(ms / 60000);
  const s = Math.floor(ms / 1000) % 60;
  return `${m}:${s.toString().padStart(2, "0")}.${(ms % 1000).toString().padStart(3, "0")}`;
}

/** The running clock as m:ss.mmm, its milliseconds floored. */
function clock(seconds: number): string {
  const s = Math.max(0, seconds);
  const m = Math.floor(s / 60);
  const rest = s - m * 60;
  const whole = Math.floor(rest);
  return `${m}:${whole.toString().padStart(2, "0")}.${Math.floor((rest - whole) * 1000).toString().padStart(3, "0")}`;
}

/** A delta in seconds, signed with the ASCII minus the style's faces hold: `-0.412`, `+1.050`. */
function deltaText(seconds: number): string {
  const a = Math.abs(seconds);
  return `${seconds < 0 ? "-" : "+"}${a >= 60 ? lapTime(a) : a.toFixed(3)}`;
}

/** A delta's ink: ahead (negative) success, behind danger, even muted. */
function deltaInk(seconds: number): Paint {
  return token(seconds < 0 ? "success" : seconds > 0 ? "danger" : "inkMuted");
}

/** The style's HUD chip under a group of the HUD's text. */
function chip(t: ReturnType<typeof tokens>) {
  return { background: token("hud"), frame: "hud" as const, padding: xy(t.md, t.xs), radius: t.control };
}

/** The HUD: what it shows from `desc`, Pause (or Back) and the touch controls' answers. */
export function racingHud(id: string, desc: HudDesc): HudOut {
  const t = tokens();
  og.ui.setNavigation(false);
  const out: HudOut = { paused: og.ui.back(), moveX: 0, held: [] };
  const showTouch = desc.touch && t.touch;
  og.ui.anchor(`${id}.place`, { anchor: "topLeft", gap: t.lg }, () => {
    og.ui.column("Lap", { ...chip(t), gap: 0, cross: "start" }, () => {
      og.ui.label("Value", `${Math.min(desc.lap, desc.laps)}/${desc.laps}`, { step: "xl3", font: "mono", nowrap: true, align: "start" });
      og.ui.label("Caption", desc.lapLabel, { step: "meta", color: token("inkMuted"), nowrap: true, align: "start" });
    });
  });
  og.ui.anchor(`${id}.times`, { anchor: "topRight", direction: "column", cross: "end", gap: t.xxs }, () => {
    og.ui.row("Top", { gap: t.sm, cross: "center" }, () => {
      og.ui.row("Clock", { padding: xy(t.md, 0), height: px(t.controlMd), gap: t.sm, cross: "center", background: token("hud"), radius: t.control, frame: "hud" }, () => {
        og.ui.label("Time", clock(desc.time), { step: "xl", font: "mono", nowrap: true });
      });
      if (og.ui.actionButton("Pause", desc.pauseLabel, { size: 44, icon: "pause" }).clicked) out.paused = true;
    });
    const { delta, best, splits } = desc;
    if (delta !== null || best !== null || splits.length > 0) og.ui.column("Laps", { ...chip(t), gap: t.xxs, cross: "end" }, () => {
      if (delta !== null) og.ui.label("Delta", deltaText(delta), { step: "lg", font: "mono", color: deltaInk(delta), nowrap: true });
      if (best !== null) og.ui.label("Best", desc.bestLabel.replaceAll("{best}", lapTime(best)), { step: "meta", font: "mono", color: token("inkMuted"), nowrap: true });
      if (splits.length > 0) {
        og.ui.column("Splits", { gap: 0, cross: "end" }, () => {
          splits.forEach((sp, i) => {
            og.ui.row(og.ui.idi("Split", i), { gap: t.sm, cross: "center" }, () => {
              og.ui.label("Label", sp.label, { step: "meta", color: token("inkMuted"), nowrap: true });
              og.ui.label("Time", sp.time === null ? "-:--.---" : lapTime(sp.time), { step: "meta", font: "mono", color: token(sp.time === null ? "inkFaint" : "ink"), nowrap: true });
              if (sp.delta !== null) og.ui.label("Delta", deltaText(sp.delta), { step: "meta", font: "mono", color: deltaInk(sp.delta), nowrap: true });
            });
          });
        });
      }
    });
    og.ui.minimap("Map", { ...desc.map, size: t.phone ? 112 : desc.map.size });
  });
  og.ui.anchor(`${id}.dash`, { ...chip(t), anchor: showTouch ? "bottom" : "bottomRight", offsetY: showTouch ? -120 : 0, direction: "column", cross: "end", gap: t.xs, width: px(220) }, () => {
    og.ui.row("Speed", { gap: t.sm, cross: "end", width: grow, align: "end" }, () => {
      og.ui.label("Value", `${Math.round(desc.speed)}`, { step: "displayXl2", font: "display", nowrap: true });
      og.ui.column("Side", { cross: "end" }, () => {
        og.ui.label("Gear", desc.gear, { step: "xl", font: "mono", color: token("highlight"), nowrap: true });
        og.ui.label("Unit", desc.unit, { step: "meta", color: token("inkMuted"), nowrap: true });
      });
    });
    og.ui.meter("Rev", desc.speed, desc.maxSpeed, { kind: "primary", width: grow, height: 6 });
  });
  if (showTouch) {
    og.ui.anchor("Steer", { anchor: "bottomLeft", gap: t.md }, () => {
      const l = og.ui.actionButton("left", desc.leftLabel, { size: 72 });
      const r = og.ui.actionButton("right", desc.rightLabel, { size: 72 });
      out.moveX = (r.pressed ? 1 : 0) - (l.pressed ? 1 : 0);
    });
    og.ui.anchor("Pedals", { anchor: "bottomRight", gap: t.md, cross: "end" }, () => {
      if (og.ui.actionButton("brake", desc.brakeLabel, { primary: false, size: 64 }).pressed) out.held.push("brake");
    });
  }
  return out;
}
