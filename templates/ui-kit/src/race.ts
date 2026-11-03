// The racing showcase's HUD (docs/ui.md §17, the art director's racing
// brief of 2026-11-03, benchmarked on Forza Horizon, Gran Turismo, F1, Need
// for Speed and Mario Kart) over the engine-rendered circuit of
// src/raceScene.ts, composed from og-ui's racing parts: the lap timer's
// head on the scene top start (the place, the lap, its sector pips) over
// the relative standings tower, its times top centre, the circuit player-up
// top end and the tach bottom end, driven by revs from a scripted gear
// model; the race's moments through og.ui.raceBanner. The arcade packs
// (cartoon, kawaii) wear the arcade look of the same parts (the owner,
// 2026-11-03): the huge ordinal, coins, the item slot top end with the map
// moved bottom start, the arcade speedometer's nitro tube and a drift
// chain. pixel keeps the digital readout. `sc:race:<event>` plays a moment
// for the playtests (countdown, checkpoint, wrongWay, finalLap, finish).

import { og } from "@origingame/engine";
import type { LapTimerDesc, MapMarkerItem, Paint, RaceEvent, SectorState, StandingsEntry, Vector2, World } from "@origingame/engine";
import { L } from "./patterns.ts";
import { frame, hudArt, type HudArt } from "./huds.ts";
import { pose, raceScene, rivalDistance, RIVALS, type RaceLight } from "./raceScene.ts";
import { mapPoint, SAMPLES } from "./raceTrack.ts";

const rgba = (r: number, g: number, b: number, a = 1): Paint => ({ color: { r, g, b, a } }) as Paint;

// ── the race's state ────────────────────────────────────────────────────

/** A moment the playtests play: the banner's event, or the wrong way over it. */
type Moment = RaceEvent | "wrongWay";
let moment: Moment = "none";
/** The tick the moment started; -1 until the HUD next builds (commands run outside a frame). */
let since = -1;

export function resetRace(): void {
  moment = "none";
  since = -1;
}

/** `sc:race:<event>`: play a race moment from the next frame. */
export function raceCommand(args: readonly string[]): boolean {
  if (args[0] !== "race") return false;
  const e = args[1] ?? "none";
  if (!["none", "countdown", "go", "checkpoint", "finalLap", "finish", "wrongWay"].includes(e)) return false;
  moment = e as Moment;
  since = -1;
  return true;
}

// ── the car: a scripted gear model ──────────────────────────────────────

/** km/h per 1,000 rpm in each gear: a road car's six-speed box. */
const RATIOS = [0, 11.5, 17.5, 24, 30, 36, 42];
const MAX_RPM = 8000;
const SHIFT_RPM = 7400;

/** The speed swinging through a lap's corners and straights, the gear the lowest under the shift point, the revs from both. */
function drive(tick: number, still: boolean): { speed: number; gear: string; rpm: number } {
  if (still) return { speed: 0, gear: "N", rpm: 0.12 + 0.05 * Math.sin(tick / 9) };
  const speed = 222 + 46 * Math.sin(tick / 156);
  let g = 1;
  while (g < 6 && (speed / RATIOS[g]!) * 1000 > SHIFT_RPM) g++;
  return { speed, gear: String(g), rpm: (speed / RATIOS[g]!) * 1000 / MAX_RPM };
}

// ── the field and the circuit ───────────────────────────────────────────

/** The field in race order, each car's gap to the leader in seconds. */
const FIELD: ReadonlyArray<[string, number]> = [["Nyx", 0], ["Ferro", 1.985], ["You", 2.861], ["Lumen", 3.482], ["Quill", 4.169], ["Sable", 7.88], ["Orrin", 9.02], ["Brask", 12.4]];
const PLACE = 3;
const HUES: Paint[] = [rgba(0.86, 0.16, 0.18), rgba(0.12, 0.42, 0.86), rgba(0.98, 0.56, 0.12), rgba(0.18, 0.7, 0.42), rgba(0.6, 0.3, 0.86), rgba(0.96, 0.46, 0.14), rgba(0.5, 0.52, 0.58), rgba(0.1, 0.72, 0.8)];

/** The circuit on the map: every tenth sample of the scene's road (src/raceTrack.ts), in map coordinates. */
const TRACK: Vector2[] = SAMPLES.filter((_, i) => i % 10 === 0).map((p) => mapPoint(p.x, p.z));

/** The map's markers: the player and the rivals where the scene drives them, the start line; the heading the player's. */
function markers(d: number): { me: { x: number; y: number; heading: number }; list: MapMarkerItem[] } {
  const mark = (dd: number, side: number) => {
    const p = pose(dd, side);
    return { ...mapPoint(p.position.x, p.position.z), heading: (p.heading * 180) / Math.PI };
  };
  const me = mark(d, 0.6);
  const list: MapMarkerItem[] = [
    { key: "me", kind: "player", x: me.x, y: me.y, heading: me.heading, label: L("sc.you") },
    ...RIVALS.map(([, side], i): MapMarkerItem => {
      const p = mark(rivalDistance(d, i), side);
      return { key: FIELD[1 - i]![0], kind: "enemy", x: p.x, y: p.y, label: FIELD[1 - i]![0], color: HUES[1 - i] ?? null, ringed: i === 0 } as MapMarkerItem;
    }),
  ];
  const start = mapPoint(SAMPLES[0]!.x, SAMPLES[0]!.z);
  list.push({ key: "start", kind: "objective", x: start.x, y: start.y, label: L("sc.race.finish") } as MapMarkerItem);
  return { me, list };
}

// ── look ────────────────────────────────────────────────────────────────

type Look = "sim" | "arcade" | "pixel";

function lookOf(style: string): Look {
  return style === "cartoon" || style === "kawaii" ? "arcade" : style === "pixel" ? "pixel" : "sim";
}

/** The scene's light by the pack: noon for the arcade packs, golden hour for broadcast, else night. */
function lightOf(style: string): RaceLight {
  return style === "cartoon" || style === "kawaii" ? "noon" : style === "broadcast" ? "dusk" : "night";
}

/** The tick the HUD was last built, and the tick it was shown from: the car drives from the same place each time it shows. */
let lastBuilt = -10;
let shownAt = 0;

// ── the HUD ─────────────────────────────────────────────────────────────

/**
 * On a phone in portrait the kit's chrome takes the top safe area (showcases.ts' CHROME, off the road): how far the
 * HUD's top anchors drop to start under it, from the chrome as laid out last tick (it grows with the text size).
 */
function chromeDrop(m: number): number {
  const vp = og.ui.frameViewport();
  if (vp.breakpoint !== "phonePortrait") return 0;
  // Only the chrome laid out in the top band counts: on the first tick its bounds are the screen before's (elsewhere).
  const b = og.ui.bounds("Back");
  if (!b || b.y > vp.height * 0.25) return 0;
  return Math.max(0, Math.round(b.y + b.height + og.ui.frameTheme().space.sm - (vp.safeArea.top + m)));
}

/** This lap's state, shared by the HUD's parts. */
interface Lap {
  lap: number;
  final: boolean;
  lapMs: number;
  delta: number | null;
  sectors: SectorState[];
}

/**
 * A racing game's HUD (Forza Horizon, Gran Turismo, F1; Mario Kart in the
 * arcade packs). Sim: the place and lap on the scene top start over the
 * relative tower, the lap times and split top centre, the circuit top end,
 * the tach bottom end. On a phone in portrait one bar across the top, the
 * tower and the map under it, the tach centred over the thumbs' pads.
 */
export function raceHud(world: World): void {
  void world;
  const { small, portrait, touch, k, m } = frame();
  og.ui.setNavigation(false);
  const style = og.ui.frameTheme().style;
  const look = lookOf(style);
  const big = og.ui.frameTheme().textScale >= 1.5;
  const tick = og.ui.tick();
  if (tick - lastBuilt > 1) shownAt = tick;
  lastBuilt = tick;
  if (since < 0) since = tick;
  const age = tick - since;
  const starting = moment === "countdown" || moment === "go";
  const final = moment === "finalLap" || moment === "finish";
  const lap: Lap = {
    lap: starting ? 1 : final ? 3 : 2,
    final,
    lapMs: starting ? 0 : 73_217 + (moment === "none" ? (tick * (1000 / 60)) % 9000 : age * (1000 / 60)),
    // No split before the start: the lap has not begun.
    delta: starting ? null : moment === "checkpoint" ? -420 : -420 + ((tick / 60) % 5) * 60,
    sectors: starting ? ["none", "none", "none"] : ["best", "personal", "none"],
  };
  const car = drive(tick, starting);
  // Metres driven since the HUD showed, from a place before the banked sweeper (the speed's integral); on the grid at the start.
  const run = tick - shownAt;
  const distance = starting ? -10 : 100 + ((222 * run) / 60 - 46 * (156 / 60) * (Math.cos(tick / 156) - Math.cos(shownAt / 156))) / 3.6;
  raceScene({ light: lightOf(style), distance, kmh: car.speed, portrait });
  const pos = markers(distance);

  if (look === "arcade") arcade(small, portrait, touch, k, m, lap, car.speed);
  else if (portrait) phoneTop(m, lap, big, pos);
  else {
    // On a phone in landscape, and at a large text size on a narrow screen (a tablet), the times join the place's column,
    // clear of the top centre.
    const stacked = small || (big && og.ui.frameViewport().width < 1100);
    og.ui.anchor("Race", { anchor: "topLeft", margin: m, direction: "column", gap: og.ui.frameTheme().space.sm }, () => {
      og.ui.lapTimer("Head", timer(lap, "head", true));
      if (stacked) og.ui.lapTimer("Times", timer(lap, "times", false));
      // 2 ahead and 2 behind (og-ui keeps a phone to 1 and 1); at a large text size on a phone the tower drops first.
      if (!(small && big)) tower();
    });
    if (!stacked) og.ui.lapTimer("Times", { ...timer(lap, "times", false), anchor: "top" });
    map(Math.round((small ? 132 : 196) * k), "topRight", "square", pos);
  }
  if (look === "arcade") {
    og.ui.anchor("ArcadeMap", arcadeMapPlace(small, portrait, touch, k, m), () => map(Math.round((small ? 120 : 176) * k), null, "circle", pos));
  } else {
    // Bottom end; on a phone in portrait centred over the pads (the camera raises the car clear of it), on a phone on
    // its side beside the pedals (the bottom centre is the car's), from the pedals as laid last tick.
    const pedals = small && touch && !portrait ? og.ui.bounds("Pedals") : null;
    const instrument = portrait ? { anchor: "bottom" as const, margin: m, offsetY: -Math.round(124 * k) }
      : small && touch ? { anchor: "bottomRight" as const, margin: m, offsetX: -Math.round((pedals?.width ?? 188) + og.ui.frameTheme().space.md) }
      : { anchor: "bottomRight" as const, margin: m };
    og.ui.anchor("Instrument", instrument, () => og.ui.speedometer("Speed", {
      style: look === "pixel" ? "digital" : "tach",
      speed: car.speed, maxSpeed: 340, unit: L("sc.race.kmh"), rpm: car.rpm, maxRpm: MAX_RPM, rpmStep: 1000, redline: 0.85, shiftAt: SHIFT_RPM / MAX_RPM,
      gear: car.gear, boost: 0.62, size: Math.round((portrait ? 176 : small ? 168 : 236) * k),
      anchor: null,
    }));
  }
  if (touch) pads(k, m, small, look === "arcade");

  // The checkpoint and the final lap sit under the times plate, wherever this layout put it.
  const below = look === "arcade" ? "" : portrait ? "Race/Bar/Times" : small || (big && og.ui.frameViewport().width < 1100) ? "Race/Times" : "TimesHud/Times";
  og.ui.raceBanner("Banner", {
    below,
    event: moment === "wrongWay" ? "none" : moment, at: since, wrongWay: moment === "wrongWay",
    count: 3, deltaMs: -420, index: 2, best: false,
    place: PLACE, placeText: L("sc.race.ord3"), timeMs: 3 * 60_000 + 41_287,
    goLabel: L("sc.race.go"), checkpointLabel: L("sc.race.checkpoint"), wrongWayLabel: L("sc.race.wrongWay"), finalLapLabel: L("sc.race.final"), finishLabel: L("sc.race.finish"),
  });
}

/** The lap timer's part: the head (the place, the lap, its pips) on the scene, or the times on their plate. */
function timer(l: Lap, show: "head" | "times", boxless: boolean): LapTimerDesc {
  return {
    show, boxless, compact: show === "times", position: PLACE, positionTotal: FIELD.length, lap: l.lap, lapTotal: 3,
    currentMs: l.lapMs, bestMs: 72_904, lastMs: 73_377, deltaMs: l.delta, sectors: l.sectors,
    positionLabel: L("sc.race.pos"), lapLabel: L("sc.race.lap"), finalLabel: L("sc.race.final"), currentLabel: L("sc.race.current"),
    bestLabel: L("sc.race.best"), lastLabel: L("sc.race.last"), splitLabel: L("sc.race.split"),
    anchor: null,
  };
}

/** The relative tower: 2 ahead and 2 behind, intervals neutral, the player's row lit. */
function tower(): void {
  const you = FIELD.find(([n]) => n === "You")![1];
  const entries: StandingsEntry[] = FIELD.map(([name, gap], i) => ({ key: name, name: name === "You" ? L("sc.you") : name, color: HUES[i], gap: gap - you, player: name === "You" }));
  og.ui.standingsTower("Standings", { entries, relative: true, ahead: 2, behind: 2, leaderLabel: L("sc.race.leader"), pitLabel: L("sc.race.pit"), outLabel: L("sc.race.out"), fastestLabel: L("sc.race.fastest"), anchor: null });
}

/**
 * The circuit, player-up round the player (og-ui centres a track's map on the player marker), its start line checkered and its sectors
 * ticked, its name under it.
 */
function map(size: number, anchor: "topRight" | "bottomLeft" | null, shape: "square" | "circle", pos: ReturnType<typeof markers>, caption = true): void {
  og.ui.minimapView("Track", { size, shape, heading: pos.me.heading, zoom: 0.62, track: TRACK, sectors: [0.36, 0.64], markers: pos.list, zone: caption ? L("sc.race.circuit") : "", north: "N", anchor });
}

/** On a phone in portrait: one bar across the top (the place and lap, the times), the tower and the map under it. */
function phoneTop(m: number, lap: Lap, big: boolean, pos: ReturnType<typeof markers>): void {
  const t = og.ui.frameTheme();
  const k = frame().k;
  og.ui.anchor("Race", { anchor: "top", margin: m, offsetY: chromeDrop(m), direction: "column", gap: t.space.sm, width: { kind: "grow" } }, () => {
    og.ui.row("Bar", { width: { kind: "grow" }, align: "spaceBetween", cross: "center", padding: { left: t.space.md, top: t.space.xs, right: t.space.md, bottom: t.space.xs }, wrap: true }, () => {
      og.ui.setDraw({ shape: { kind: "rect", radii: Array(4).fill(t.radius["panel"] ?? 10) }, fills: [{ kind: "solid", color: { token: "hud", alpha: 0.82 } }] } as never);
      og.ui.lapTimer("Head", timer(lap, "head", true));
      og.ui.lapTimer("Times", timer(lap, "times", true));
    });
    og.ui.row("Under", { width: { kind: "grow" }, align: "spaceBetween", cross: "start" }, () => {
      // At a large text size the tower drops before the instrument shrinks, and the map shrinks and loses its caption.
      if (!big) tower();
      else og.ui.column("Room", { width: { kind: "fixed", px: 1 } }, () => {});
      map(Math.round((big ? 84 : 124) * k), null, "circle", pos, !big);
    });
  });
}

/**
 * The touch pads (the brief's 72, 64 and 84 pt): steer as two chevrons, brake and boost round, each in a full state
 * ring (a faint track, the value lit along it). The atlas's back chevron, mirrored for Right, is in every pack.
 */
function pads(k: number, m: number, small: boolean, arcadeLook: boolean): void {
  const t = og.ui.frameTheme();
  // On a tablet the kit's chrome takes the bottom start corner: the steering sits over it.
  og.ui.anchor("Steer", { anchor: "bottomLeft", margin: m, direction: "row", gap: 2, offsetY: small ? 0 : -Math.round(64 * k * t.textScale) }, () => {
    og.ui.actionButton("Left", L("sc.race.left"), { icon: "back", size: 72 });
    og.ui.actionButton("Right", L("sc.race.right"), { icon: "back", size: 72 });
    og.ui.props({ transform: { scaleX: -1 } } as never);
  });
  const ring = (id: string, side: number, value: number, ink: string, build: () => void) => {
    const d = side + 16;
    const arc = (_: number, color: Record<string, unknown>) => ({ kind: "solid", color });
    og.ui.column(id, { width: { kind: "fixed", px: d }, height: { kind: "fixed", px: d }, align: "center", cross: "center" }, () => {
      og.ui.setDraw({ shape: { kind: "arc", start: -Math.PI / 2, sweep: Math.PI * 2, thickness: 5, cap: "butt" }, fills: [arc(1, { token: "shadow", alpha: 0.55 })] } as never);
      og.ui.column("Lit", { width: { kind: "fixed", px: d }, height: { kind: "fixed", px: d } }, () => {});
      og.ui.setDraw({ shape: { kind: "arc", start: -Math.PI / 2, sweep: Math.PI * 2 * Math.max(0.001, value), thickness: 5, cap: "round" }, fills: [arc(1, { token: ink })] } as never);
      og.ui.props({ position: "absolute", insets: { start: 0, top: 0 }, inert: true } as never);
      // The pad itself round inside its ring: the button clipped to a circle.
      og.ui.column("Pad", { width: { kind: "fixed", px: side }, height: { kind: "fixed", px: side }, align: "center", cross: "center" }, build);
      og.ui.setDraw({ shape: { kind: "ellipse" }, clip: "shape" } as never);
    });
  };
  // On a phone in the bottom end corner (the instrument centred); on a tablet over the instrument in that corner.
  og.ui.anchor("Pedals", { anchor: "bottomRight", margin: m, direction: "row", gap: t.space.sm, cross: "end", offsetY: small || arcadeLook ? 0 : -Math.round(250 * k) }, () => {
    ring("BrakeRing", 64, 0.35, "danger", () => og.ui.actionButton("Brake", L("sc.race.brake"), { size: 64 }));
    ring("BoostRing", 84, 0.7, "info", () => og.ui.actionButton("Boost", L("sc.race.boost"), { primary: true, icon: "bolt", size: 84 }));
  });
}

// ── the arcade look ─────────────────────────────────────────────────────

/** Where the arcade map sits: bottom start over the kit's chrome and the steering, or under the item slot on a phone in portrait. */
function arcadeMapPlace(small: boolean, portrait: boolean, touch: boolean, k: number, m: number) {
  const lift = small ? 84 : (touch ? 148 : 56) * og.ui.frameTheme().textScale;
  return portrait ? { anchor: "topRight" as const, margin: m, offsetY: Math.round(92 * k) + chromeDrop(m) } : { anchor: "bottomLeft" as const, margin: m, offsetY: -Math.round(lift * k) };
}

/** Mario Kart's read on the same parts: the huge ordinal over its lap top start, coins, the item slot top end, the arcade speedometer's nitro tube bottom end, a drift chain by the car. */
function arcade(small: boolean, portrait: boolean, touch: boolean, k: number, m: number, lap: Lap, speed: number): void {
  const t = og.ui.frameTheme();
  const tick = og.ui.tick();
  const coins = () => og.ui.row("Coins", { gap: t.space.xs, cross: "center", padding: { left: t.space.sm, top: 2, right: t.space.md, bottom: 2 } }, () => {
    const r = 999;
    og.ui.setDraw({ shape: { kind: "rect", radii: [r, r, r, r] }, fills: [{ kind: "solid", color: { token: "panel", alpha: 0.94 } }], strokes: [{ width: 2.5, color: { token: "lineStrong" } }], effects: [{ kind: "dropShadow", offset: { x: 0, y: 3 }, blur: 6, spread: 0, color: { token: "shadow", alpha: 0.5 } }] } as never);
    og.ui.icon("Coin", "coin", { size: Math.round(26 * k), tint: { token: "gold" } } as never);
    og.ui.label("Count", `\u00d7 ${12 + (Math.floor(tick / 240) % 3)}`, { step: "lg", font: "display", nowrap: true });
  });
  og.ui.anchor("Race", { anchor: "topLeft", margin: m, offsetY: chromeDrop(m), direction: "column", gap: t.space.xs, cross: "start" }, () => {
    og.ui.lapTimer("Head", { ...timer(lap, "head", true), placeBadge: true, positionText: L("sc.race.ord3") });
    // On a phone the coins join the column (the top centre is the kit's chrome in landscape); else they sit top centre.
    if (small) coins();
  });
  if (!small) og.ui.anchor("Top", { anchor: "top", margin: m }, coins);
  const bolt = hudArt("race/item-bolt" as HudArt);
  og.ui.anchor("ItemHud", { anchor: "topRight", margin: m, offsetY: chromeDrop(m) }, () => og.ui.itemSlot("Item", { item: bolt, label: L("sc.race.boost"), size: Math.round((small ? 64 : 84) * k), anchor: null }));
  // A drift chain over the car (the screen's centre, a little right and up); left out on a phone at a large text size.
  const vp = og.ui.frameViewport();
  // By the car; in portrait under the round map (as laid last tick), whose column it would otherwise cross.
  const mapB = portrait ? og.ui.bounds("ArcadeMap") : null;
  const driftH = og.ui.bounds("DriftHud")?.height ?? 60;
  const driftY = Math.round(Math.max(vp.height * (portrait ? 0.37 : 0.46), mapB ? mapB.y + mapB.height + t.space.sm + driftH : 0));
  if (!(small && t.textScale >= 1.5)) og.ui.driftScore("Drift", { score: 2450, multiplier: 3, total: 18_200, label: L("sc.race.drift"), at: { x: Math.round(vp.width * (portrait ? 0.7 : 0.66)), y: driftY } });
  og.ui.anchor("Instrument", { anchor: "bottomRight", margin: m, offsetY: touch ? -Math.round(112 * k) : 0 }, () => og.ui.speedometer("Speed", {
    style: "arcade", speed: speed * 0.62, maxSpeed: 200, unit: L("sc.race.kmh"), boost: 0.7, boostSegments: 4, size: Math.round((small ? 150 : 200) * k), anchor: null,
  }));
}

/** The results after the finish: the results document with the race's line and stats. */
export function raceResultsViewmodel(): Record<string, unknown> {
  return {
    won: true, grade: "", title: L("sc.race.podium"), tab: "board", level: 14, xp: 0.38, gained: 860,
    stats: [{ name: L("sc.race.overtakes"), value: 5, best: false }, { name: L("sc.race.topSpeed"), value: 291, best: true }, { name: L("sc.race.clean"), value: 7, best: false }],
    rewards: [{ name: L("sc.race.credits"), amount: 1200, texture: hudArt("boons/gold") ?? 0 }],
  };
}

/** The race's classification (the results' board): place, driver, race time, best lap, gap; the fastest lap purple, the player's row lit. */
const CLASSIFIED: ReadonlyArray<[string, number, number]> = [
  ["Nyx", 219_402, 72_118], ["Ferro", 221_387, 72_904], ["You", 221_287, 71_861], ["Lumen", 222_884, 73_012], ["Quill", 223_571, 73_240], ["Sable", 227_282, 73_910],
];

export function raceClassification(): void {
  const t = og.ui.frameTheme();
  const rows = [...CLASSIFIED].sort((a, b) => a[1] - b[1]);
  const fastest = Math.min(...rows.map((r) => r[2]));
  const leader = rows[0]![1];
  const w = (n: number) => ({ kind: "fixed" as const, px: Math.round(n * Math.min(1.4, t.textScale)) });
  og.ui.column("Classification", { width: { kind: "grow" }, gap: 2 }, () => {
    rows.forEach(([name, total, best], i) => {
      const you = name === "You";
      og.ui.row(name, { width: { kind: "grow" }, gap: t.space.md, cross: "center", padding: { left: t.space.sm, top: t.space.xs, right: t.space.sm, bottom: t.space.xs } }, () => {
        if (you) og.ui.setDraw({ shape: { kind: "rect", radii: Array(4).fill(t.radius["control"] ?? 6) }, fills: [{ kind: "solid", color: { token: "primary" } }] } as never);
        const ink = you ? ({ token: "onPrimary" } as Paint) : null;
        const muted = you ? ink : ({ token: "inkMuted" } as Paint);
        og.ui.label("Place", String(i + 1), { step: "lg", font: "display", color: ink, nowrap: true, width: w(28), align: "end" });
        og.ui.label("Name", you ? L("sc.you") : name, { step: "base", font: you ? "display" : "body", color: ink, nowrap: true, ellipsis: true, width: { kind: "grow" } });
        og.ui.label("Total", lapTime(total), { step: "base", font: "mono", color: ink, nowrap: true });
        og.ui.label("Best", lapTime(best), { step: "sm", font: "mono", color: best === fastest ? FASTEST : muted, nowrap: true });
        og.ui.label("Gap", i === 0 ? "" : `+${((total - leader) / 1000).toFixed(3)}`, { step: "sm", font: "mono", color: muted, nowrap: true, width: w(64), align: "end" });
      });
    });
  });
}

/** The fastest lap's purple, as F1 timing marks it. */
const FASTEST = rgba(0.68, 0.42, 1);

/** A time as racing games write it: m:ss.mmm. */
function lapTime(ms: number): string {
  const t = Math.round(ms);
  return `${Math.floor(t / 60000)}:${String(Math.floor((t % 60000) / 1000)).padStart(2, "0")}.${String(t % 1000).padStart(3, "0")}`;
}
