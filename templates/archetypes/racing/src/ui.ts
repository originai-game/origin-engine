// Redline Ring's screens on og.ui and its style (assets/styles/redline, a
// copy of the engine's broadcast pack): the title over the field's attract
// laps and the pause (the game's screen documents, assets/ui/main-menu.ogt
// and pause.ogt), the race's HUD from the racing kit (the place and lap,
// the lap times and the split, the standings, the speedometer with its
// boost meter, the race banner's countdown, checkpoint, final lap, finish
// and wrong way) beside og.ui's minimap of the circuit, the touch
// controls (Left and Right, Brake, Boost and Throttle), the settings
// dialog and the results (the kit's race-results document, ejected to
// assets/ui/racing/). Screens change through a dissolve (a fade with
// reduced motion). Every string is a key of assets/strings/*.ogt.
//
// Every choice that changes the world reaches it through a seat's input
// (src/seats.ts press), so a room makes it on one tick on every peer: Race
// and Race again press Throttle, Retire and Title press Menu (a control
// with no key of its own), Pause and Resume press Pause.
//
// The Ui system also starts races, holds the world while paused, moves to
// the results once the race has closed (src/race.ts), sets the buses and
// keeps the settings and the record (this device's, src/device.ts) in
// og.platform: the record written at the revision read, and merged when
// another device wrote it first.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { racing, type RaceBannerDesc, type SectorState, type StandingsEntry } from "@origingame/engine/extensions/racing/index.ts";
import { Car, COUNTDOWN, FIELD, Game, Phase, RESULTS_DELAY, Sound } from "./components.ts";
import { CARS } from "./car.ts";
import { prefs, record, screens } from "./device.ts";
import { bit, LOCAL, localSeats, ONLINE, press, pressedNow, SEATS, usesTouch, useTouchControls } from "./seats.ts";
import { LAPS, SAMPLES, SECTORS } from "./track.ts";
import { LIVERIES, must, slotOf, toGrid } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** Ticks from the results showing until Throttle and Menu answer there. */
export const RESULTS_READY = 40;
/** Points by place, first to sixth. */
export const POINTS = [10, 8, 6, 4, 2, 1] as const;

/** The tokens' spacing scale in CSS px. */
const S = { xs: 4, sm: 8, md: 12, lg: 16 } as const;

type GameV = Value<typeof Game>;
type CarV = Value<typeof Car>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;
const ms = (ticks: number) => Math.round((ticks * 1000) / 60);

/** m:ss.mmm of a count of ticks. */
export function lapText(ticks: number): string {
  const total = ms(ticks);
  const m = Math.floor(total / 60000);
  const s = Math.floor((total % 60000) / 1000);
  const f = total % 1000;
  return `${m}:${s < 10 ? "0" : ""}${s}.${f < 10 ? "00" : f < 100 ? "0" : ""}${f}`;
}

/** The field's rows, in grid order. */
function field(world: World): CarV[] {
  return CARS.map((e) => must(world.get(e, Car)));
}

/** A car's driver as the screens name them: you alone, P1 and P2 with two seats, the rivals by name. */
function driverName(world: World, c: CarV): string {
  if (c.seat < 0) return LIVERIES[c.index]!.name;
  return SEATS > 1 ? say(world, "name.player", { n: c.seat + 1 }) : say(world, "name.you");
}

/** A new race from the grid: the countdown runs, the record counts it. */
export function startRace(world: World, g: GameV): void {
  g.phase = Phase.Race;
  g.paused = false;
  g.clock = 0;
  g.firstFinish = 0;
  g.lastSeatFinish = 0;
  g.closed = false;
  g.bumps = 0;
  g.start = true;
  CARS.forEach((e, i) => toGrid(world, e, slotOf(i, SEATS)));
  record.races += 1;
  record.newBest = false;
  saveRecord(world);
}

/** Back to the title: the field on the grid again, lapping behind it on the autopilot. */
export function toTitle(world: World, g: GameV): void {
  g.phase = Phase.Title;
  g.paused = false;
  g.start = true;
  // Behind the title the seats lead the field away (open road ahead of the camera); a race puts them at the back.
  CARS.forEach((e, i) => toGrid(world, e, i));
}

/** Whether this device plays in a running room (its record and settings' hold are then its own business). */
function inRoom(world: World): boolean {
  const lock = world.resource(og.Lockstep);
  return ONLINE && lock.ok && lock.value.running;
}

/**
 * The record ({races, wins, bestLap}) written at the revision last read
 * or written. One call at a time: while a read or a write is in flight
 * the newer record waits (dirty) and goes out when that answers. A running
 * room keeps none: a peer's own store answers come back on its own ticks.
 */
function saveRecord(world: World): void {
  if (inRoom(world)) return;
  if (record.request !== 0 || record.write !== 0) {
    record.dirty = true;
    return;
  }
  record.dirty = false;
  const r = og.platform.storeSet(world, { key: "record", value: { races: record.races, wins: record.wins, bestLap: record.bestLap }, revision: record.revision });
  if (!r.ok) return;
  record.write = r.value;
  record.sentRaces = record.races;
  record.sentWins = record.wins;
}

function applyPrefs(world: World): void {
  const audio = world.resource(og.Audio);
  if (audio.ok) {
    audio.value.master = prefs.master;
    audio.value.music = prefs.music;
    audio.value.sfx = prefs.effects;
    audio.value.ui = prefs.effects;
  }
}

function setLanguage(world: World, index: number): void {
  prefs.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[prefs.language]!.code);
}

function prefsValue() {
  return { masterVolume: prefs.master, musicVolume: prefs.music, sfxVolume: prefs.effects, language: LANGUAGES[prefs.language]!.code, reducedMotion: prefs.reducedMotion };
}

/**
 * og.platform: the settings and the record asked for on tick 0 and
 * applied when they answer; a record write refused for a stale revision
 * (another device raced meanwhile) is read again, merged (the stored counts
 * plus what this device added since it last read or wrote them, the
 * better of the two best laps) and written at the new revision.
 */
function platform(world: World): void {
  if (world.tick === 0) {
    prefs.loadRequest = og.platform.loadSettings(world);
    if (!inRoom(world)) {
      const r = og.platform.storeGet(world, "record");
      record.request = r.ok ? r.value : 0;
    }
    return;
  }
  if (prefs.loadRequest !== 0) {
    const r = og.platform.result(world, prefs.loadRequest);
    if (r !== null) {
      prefs.loadRequest = 0;
      const s = ((r.ok ? r.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
      if (s && typeof s === "object") {
        if (typeof s.masterVolume === "number") prefs.master = s.masterVolume;
        if (typeof s.musicVolume === "number") prefs.music = s.musicVolume;
        if (typeof s.sfxVolume === "number") prefs.effects = s.sfxVolume;
        if (typeof s.reducedMotion === "boolean") prefs.reducedMotion = s.reducedMotion;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, at);
        applyPrefs(world);
      }
    }
  }
  if (inRoom(world)) return;
  if (record.request !== 0) {
    const r = og.platform.result(world, record.request);
    if (r !== null) {
      record.request = 0;
      const d = (r.ok ? r.data : null) as { value?: { races?: number; wins?: number; bestLap?: number } | null; revision?: number } | null;
      const races = d?.value?.races ?? 0;
      const wins = d?.value?.wins ?? 0;
      const best = d?.value?.bestLap ?? 0;
      record.races = races + record.races - record.storedRaces;
      record.wins = wins + record.wins - record.storedWins;
      if (best > 0 && (record.bestLap === 0 || best < record.bestLap)) record.bestLap = best;
      record.storedRaces = races;
      record.storedWins = wins;
      if (typeof d?.revision === "number") record.revision = d.revision;
      if (record.merge || record.dirty) {
        record.merge = false;
        saveRecord(world);
      }
    }
  }
  if (record.write !== 0) {
    const r = og.platform.result(world, record.write);
    if (r !== null) {
      record.write = 0;
      if (r.ok) {
        record.revision = (r.data as { revision: number }).revision;
        record.storedRaces = record.sentRaces;
        record.storedWins = record.sentWins;
        if (record.dirty) saveRecord(world);
      } else if (r.error?.code === "conflict") {
        record.conflicts += 1;
        record.merge = true;
        const again = og.platform.storeGet(world, "record");
        record.request = again.ok ? again.value : 0;
      } else if (record.dirty) saveRecord(world);
    }
  }
}

/** The race over for every seat, a beat later: the results, the record and the telemetry. */
function finishRace(world: World, g: GameV): void {
  if (g.phase !== Phase.Race || !g.closed || g.clock - g.lastSeatFinish < RESULTS_DELAY) return;
  g.phase = Phase.Results;
  const mine = field(world).filter((c) => c.seat >= 0 && (LOCAL || localSeats(world).includes(c.seat)));
  const won = mine.some((c) => c.place === 1 && c.finish > 0);
  if (won) record.wins += 1;
  for (const c of mine) {
    if (c.bestLap > 0 && (record.bestLap === 0 || c.bestLap < record.bestLap)) {
      record.bestLap = c.bestLap;
      record.newBest = true;
    }
  }
  saveRecord(world);
  const me = mine[0];
  must(og.platform.telemetry(world, "race.over", { place: me?.place ?? 0, finished: (me?.finish ?? 0) > 0, bestLap: me?.bestLap ?? 0, bumps: g.bumps }));
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

/** The circuit in the minimap's coordinates (0 to 1 over its bounds, kept square). */
const MAP = (() => {
  let minX = Infinity, maxX = -Infinity, minZ = Infinity, maxZ = -Infinity;
  for (const p of SAMPLES) {
    minX = Math.min(minX, p.x);
    maxX = Math.max(maxX, p.x);
    minZ = Math.min(minZ, p.z);
    maxZ = Math.max(maxZ, p.z);
  }
  const span = Math.max(maxX - minX, maxZ - minZ) * 1.12;
  const cx = (minX + maxX) / 2;
  const cz = (minZ + maxZ) / 2;
  const to = (x: number, z: number) => ({ x: 0.5 + (x - cx) / span, y: 0.5 + (z - cz) / span });
  const track = SAMPLES.filter((_, i) => i % 3 === 0).map((p) => to(p.x, p.z));
  return { to, track };
})();

/** og.ui's minimap of the whole circuit, every car on it (this device's ringed). */
function minimap(world: World, size: number, mine: readonly number[]): void {
  const markers = CARS.map((e) => {
    const c = must(world.get(e, Car));
    const at = must(world.get(e, og.Transform)).position;
    const p = MAP.to(at.x, at.z);
    const own = c.seat >= 0 && mine.includes(c.seat);
    const col = LIVERIES[c.index]!.color;
    return { key: `car-${c.index}`, kind: own ? ("player" as const) : ("enemy" as const), x: p.x, y: p.y, heading: -c.heading, color: { color: { r: col.r, g: col.g, b: col.b, a: 1 } }, ringed: own, label: driverName(world, c) };
  });
  og.ui.minimapView("Map", { track: MAP.track, sectors: [...SECTORS], zoom: 1, centerX: 0.5, centerY: 0.5, rotate: false, size, markers, north: "", anchor: null });
}

/** The standings: the whole field in place order, this device's drivers marked. */
function standings(world: World, mine: readonly number[], rows: number): void {
  const cars = field(world).sort((a, b) => a.place - b.place);
  const leader = cars[0]!;
  const entries: StandingsEntry[] = cars.map((c) => {
    const col = LIVERIES[c.index]!.color;
    // The gap to the leader in seconds, from the distance between them at the car's pace.
    const gap = c === leader ? 0 : c.finish > 0 && leader.finish > 0 ? (c.finish - leader.finish) / 60 : (leader.progress - c.progress) / Math.max(15, c.speed);
    return { key: `car-${c.index}`, name: driverName(world, c), color: { color: { r: col.r, g: col.g, b: col.b, a: 1 } }, gap, player: c.seat >= 0 && mine.includes(c.seat), out: c.out };
  });
  racing.standingsTower("Standings", { entries, rows, relative: false, leaderLabel: say(world, "hud.leader"), outLabel: say(world, "hud.out"), anchor: null });
}

/** The race banner's moment for a car: the latest of the countdown, a checkpoint, the final lap and the finish, or the wrong way. */
function banner(world: World, g: GameV, c: CarV | null, below: string): RaceBannerDesc {
  const uiNow = og.ui.tick();
  const ago = (clock: number) => uiNow - (g.clock - clock);
  const words = {
    goLabel: say(world, "hud.go"),
    checkpointLabel: say(world, "hud.checkpoint"),
    wrongWayLabel: say(world, "hud.wrongWay"),
    finalLapLabel: say(world, "hud.finalLap"),
    finishLabel: say(world, "hud.finish"),
    below,
  };
  const countdown: RaceBannerDesc = { ...words, event: "countdown", at: ago(0), count: 3 };
  if (c === null) return countdown;
  if (c.finish > 0) return { ...words, event: "finish", at: ago(c.finish), place: c.place, placeText: say(world, `place.${Math.min(c.place, FIELD)}`), timeMs: ms(c.finish - COUNTDOWN) };
  const moments: [number, RaceBannerDesc][] = [[0, countdown]];
  if (c.laps === LAPS - 1 && LAPS > 1) moments.push([c.lapAt, { ...words, event: "finalLap", at: ago(c.lapAt) }]);
  if (c.splitAt > 0 && c.splitDelta !== 0) moments.push([c.splitAt, { ...words, event: "checkpoint", at: ago(c.splitAt), deltaMs: ms(c.splitDelta), best: c.splitDelta < 0, index: c.checkpoint }]);
  moments.sort((a, b) => b[0] - a[0]);
  return { ...moments[0]![1], wrongWay: c.wrongWay > 45 && g.clock > COUNTDOWN };
}

/** The lap timer's two faces for a car: the place and lap, and the lap's times with its split. */
function laps(world: World, g: GameV, c: CarV, compact: boolean): void {
  const lap = Math.max(1, Math.min(LAPS, c.laps + 1));
  const running = g.clock >= COUNTDOWN && c.finish === 0;
  const sectors: SectorState[] = [0, 1, 2].map((k) => (k < Math.min(c.checkpoint, 2) ? (c.splitDelta < 0 && k === c.checkpoint - 1 ? "best" : "personal") : "none"));
  const words = { positionLabel: say(world, "hud.pos"), lapLabel: say(world, "hud.lap"), finalLabel: say(world, "hud.finalLap"), currentLabel: say(world, "hud.current"), bestLabel: say(world, "hud.best"), lastLabel: say(world, "hud.last"), splitLabel: say(world, "hud.split") };
  racing.lapTimer("Head", { ...words, show: "head", boxless: true, position: c.place, positionTotal: FIELD, lap, lapTotal: LAPS, anchor: null, compact });
  racing.lapTimer("Times", {
    ...words,
    show: "times",
    compact: true,
    currentMs: running ? ms(g.clock - c.lapStart) : c.finish > 0 ? ms(c.lastLap) : 0,
    bestMs: c.bestLap > 0 ? ms(c.bestLap) : null,
    lastMs: c.lastLap > 0 ? ms(c.lastLap) : null,
    deltaMs: c.splitDelta !== 0 ? ms(c.splitDelta) : null,
    sectors,
    anchor: null,
  });
}

/** A half's place, lap and running time in one small plate (two seats on a phone upright). */
function miniLaps(world: World, g: GameV, c: CarV, left: boolean): void {
  const lap = Math.max(1, Math.min(LAPS, c.laps + 1));
  const running = g.clock >= COUNTDOWN && c.finish === 0;
  og.ui.column("Mini", { gap: 0, cross: left ? "start" : "end", padding: { left: S.sm, top: S.xs, right: S.sm, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, () => {
    og.ui.label("Place", `${say(world, "hud.pos")} ${c.place}/${FIELD}`, { step: "lg", font: "display", nowrap: true });
    og.ui.label("Lap", `${say(world, "hud.lap")} ${lap}/${LAPS}`, { color: { token: "inkMuted" }, nowrap: true });
    og.ui.label("Time", lapText(running ? g.clock - c.lapStart : c.finish > 0 ? c.lastLap : 0), { font: "mono", nowrap: true });
  });
}

/** The speedometer: km/h, the boost meter in four segments. */
function speedo(world: World, c: CarV, size: number): void {
  racing.speedometer("Speed", { style: "arcade", speed: Math.round(Math.abs(c.speed) * 3.6), maxSpeed: 200, unit: say(world, "hud.kmh"), boost: c.boost, boostSegments: 4, size, anchor: null });
}

/** The HUD of a race: for one seat on this device, or for two in their halves of a split screen. */
function hud(world: World, g: GameV): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.width < 600;
  const touch = usesTouch();
  // A phone held sideways: no height for a column over the touch controls.
  const short = vp.height < 500;
  const mine = LOCAL ? [0, 1] : localSeats(world);
  const cars = field(world);
  og.ui.setNavigation(false);
  if (LOCAL) {
    for (const seat of [0, 1]) {
      const c = cars[seat]!;
      const left = seat === 0;
      og.ui.anchor(`Lead${seat}`, { anchor: left ? "topLeft" : "topRight", direction: "column", gap: S.xs, cross: left ? "start" : "end" }, () => {
        // On a phone the dial joins its half's corner (the bottom holds the touch controls): beside the times held
        // sideways, under them upright.
        if (short) {
          og.ui.row("Top", { gap: S.sm, cross: "start" }, () => {
            if (!left) speedo(world, c, 76);
            og.ui.column("Laps", { gap: S.xs, cross: left ? "start" : "end" }, () => laps(world, g, c, true));
            if (left) speedo(world, c, 76);
          });
        } else if (phone) {
          // Upright, each half is under 200 px wide: the kit's lap timer (220 px) does not fit, so the place, lap
          // and time stand in a plate of their own, the dial under them.
          miniLaps(world, g, c, left);
          speedo(world, c, 96);
        } else laps(world, g, c, true);
        if (c.finish > 0 || c.out) capsule("Done", () => og.ui.label("Text", c.out ? say(world, "hud.out") : say(world, "hud.finished", { place: say(world, `place.${c.place}`) }), { step: "lg", font: "display", nowrap: true }));
        if (c.wrongWay > 45 && g.clock > COUNTDOWN) capsule("Wrong", () => og.ui.label("Text", say(world, "hud.wrongWay"), { color: { token: "danger" }, nowrap: true }));
      });
      if (!phone && !short) og.ui.anchor(`Instrument${seat}`, { anchor: left ? "bottomLeft" : "bottomRight", margin: 16, offsetY: touch ? -194 : 0 }, () => speedo(world, c, touch ? 112 : 150));
    }
    og.ui.anchor("Middle", { anchor: "top", direction: "column", gap: S.sm, cross: "center" }, () => {
      if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 44 }).clicked) press("pause");
      // The countdown's lamps hang where the map sits: the map waits for them to go out.
      if (g.clock >= COUNTDOWN + 60) minimap(world, phone || short ? 84 : 150, mine);
    });
    // The countdown for both; each half's own moments are its column's words.
    if (g.clock < COUNTDOWN + 60) racing.raceBanner("Banner", banner(world, g, null, ""));
    return;
  }
  const c = cars[mine[0] ?? 0]!;
  og.ui.anchor("Lead", { anchor: "topLeft", direction: "column", gap: S.xs }, () => laps(world, g, c, phone));
  // Held sideways the map sits beside Pause, clear of the pedals.
  og.ui.anchor("Field", { anchor: "topRight", direction: short ? "row" : "column", gap: S.sm, cross: short ? "start" : "end" }, () => {
    if (short) minimap(world, 104, mine);
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) press("pause");
    if (!phone && !short) standings(world, mine, FIELD);
    if ((touch || phone) && !short) minimap(world, phone ? 110 : 150, mine);
  });
  if (!touch && !phone) og.ui.anchor("Map", { anchor: "bottomLeft", margin: 16 }, () => minimap(world, 190, mine));
  // Held sideways the car fills the bottom's middle: the dial sits beside Left and Right instead.
  if (short) og.ui.anchor("Instrument", { anchor: "bottomLeft", margin: 16, offsetX: 196 }, () => speedo(world, c, 104));
  else og.ui.anchor("Instrument", { anchor: touch ? "bottom" : "bottomRight", margin: 16, offsetY: touch ? -(phone ? 176 : 24) : 0 }, () => speedo(world, c, phone ? 120 : touch ? 150 : 190));
  racing.raceBanner("Banner", banner(world, g, c, phone ? "Lead" : ""));
}

/** What the touch controls need from the world, kept by Ui each tick (they are drawn by Seats, before Ui). */
const touchHud = { shown: false, world: null as World | null, boost: [0, 0] };
/** What answered on the ui bus on this device (a press of a touch control). */
const touchState = { held: [0, 0], taps: 0 };

/**
 * Redline Ring's touch controls for a seat, drawn by Seats while touch
 * shows and a race is on screen: Left and Right in the bottom-left corner,
 * Brake, Boost (its ring the boost meter) and Throttle in the bottom-right.
 * Two seats on one device each have the whole set, smaller, in their own
 * corner (seat 0 left, seat 1 right).
 */
function touchControls(seat: number): { x: number; y: number; lx: number; ly: number; buttons: number } {
  const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
  const world = touchHud.world;
  if (!touchHud.shown || world === null) return out;
  const vp = og.ui.frameViewport();
  const small = LOCAL || vp.width < 600;
  const steerSize = LOCAL ? 54 : small ? 70 : 84;
  const pedal = LOCAL ? 50 : small ? 62 : 72;
  const gas = LOCAL ? 58 : small ? 80 : 92;
  const boost = touchHud.boost[seat] ?? 0;
  const steer = () => {
    og.ui.row("Steer", { gap: S.md }, () => {
      if (og.ui.actionButton("Left", say(world, "hud.left"), { icon: "back", size: steerSize, round: true }).pressed) out.x -= 1;
      if (og.ui.actionButton("Right", say(world, "hud.right"), { icon: "play", size: steerSize, round: true }).pressed) out.x += 1;
    });
  };
  const pedals = () => {
    og.ui.row("Pedals", { gap: S.sm, cross: "end" }, () => {
      if (og.ui.actionButton("Brake", say(world, "hud.brake"), { size: pedal }).pressed) out.buttons |= bit("brake");
      og.ui.column("Right", { gap: S.sm, cross: "center" }, () => {
        if (og.ui.actionButton("Boost", say(world, "hud.boost"), { icon: "bolt", size: pedal, charge: boost, disabled: boost < 0.02 }).pressed) out.buttons |= bit("boost");
        if (og.ui.actionButton("Throttle", say(world, "hud.throttle"), { size: gas, primary: true }).pressed) out.buttons |= bit("throttle");
      });
    });
  };
  if (LOCAL) {
    og.ui.anchor(`Touch${seat}`, { anchor: seat === 0 ? "bottomLeft" : "bottomRight", direction: "column", gap: S.sm, cross: seat === 0 ? "start" : "end" }, () => {
      pedals();
      steer();
    });
  } else {
    og.ui.anchor("Steering", { anchor: "bottomLeft" }, steer);
    og.ui.anchor("Pedals", { anchor: "bottomRight" }, pedals);
  }
  // A control coming down answers on the ui bus (Sound.Tap).
  const held = out.buttons | (out.x !== 0 ? 1 << 14 : 0);
  if ((held & ~touchState.held[seat]!) !== 0) touchState.taps += 1;
  touchState.held[seat] = held;
  return out;
}
useTouchControls(touchControls);

/** A seat's control went down this tick (against Car.before, which Drivers moves on after Ui). */
function pressedBy(world: World, name: string): boolean {
  for (let seat = 0; seat < SEATS; seat++) {
    const car = CARS[seat]!;
    if (pressedNow(must(world.get(car, og.PlayerInput)).buttons, must(world.get(car, Car)).before, name)) return true;
  }
  return false;
}

/**
 * The seats' choices, from their input: Pause opens and closes the pause
 * menu (under the settings it closes them); on the title Throttle races;
 * paused, Menu retires to the title; on the results Throttle races again
 * and Menu goes back to the title.
 */
function choices(world: World, g: GameV): void {
  if (screens.settingsOpen) {
    if (pressedBy(world, "pause")) closeSettings(world);
    return;
  }
  if (pressedBy(world, "pause")) {
    if (g.paused) g.paused = false;
    else if (g.phase === Phase.Race && !g.closed) g.paused = true;
  }
  if (g.phase === Phase.Title) {
    if (pressedBy(world, "throttle")) startRace(world, g);
  } else if (g.phase === Phase.Race) {
    if (g.paused && pressedBy(world, "menu")) toTitle(world, g);
  } else if (g.clock - g.lastSeatFinish >= RESULTS_DELAY + RESULTS_READY) {
    if (pressedBy(world, "throttle")) startRace(world, g);
    else if (pressedBy(world, "menu")) toTitle(world, g);
  }
}

function settings(world: World): void {
  let done = false;
  const before = JSON.stringify(prefsValue());
  // Locked: Back reaches the game as the seat's Pause, which closes them (choices).
  og.ui.modal("Settings", screens.settingsOpen, { title: say(world, "settings.title"), locked: true, width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    prefs.master = slider("Master", "settings.master", prefs.master);
    prefs.music = slider("Music", "settings.music", prefs.music);
    prefs.effects = slider("Effects", "settings.effects", prefs.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), prefs.language, { horizontal: true });
    if (r.selected !== prefs.language) setLanguage(world, r.selected);
    prefs.reducedMotion = og.ui.toggle("Motion", prefs.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue()) !== before) applyPrefs(world);
  if (screens.settingsOpen && done) closeSettings(world);
}

/** The settings close (Done, or a seat's Pause: Escape, Start) and are saved. */
function closeSettings(world: World): void {
  screens.settingsOpen = false;
  must(og.platform.saveSettings(world, prefsValue()));
  prefs.saves += 1;
}

/** The results' viewmodel: this device's drivers' result, the classification, the best lap, the points. */
function resultsModel(world: World, g: GameV) {
  const cars = field(world).sort((a, b) => a.place - b.place);
  const mine = cars.filter((c) => c.seat >= 0 && (LOCAL || localSeats(world).includes(c.seat)));
  const best = mine.reduce((m, c) => (m === null || c.place < m.place ? c : m), null as CarV | null);
  const winner = cars[0]!;
  const title = best === null ? "results.complete"
    : best.out ? "results.dnf"
    : best.place === 1 ? (LOCAL ? "results.winPlayer" : "results.win")
    : best.place <= 3 ? "results.podium" : "results.complete";
  const rows = cars.map((c) => ({
    place: c.place,
    name: driverName(world, c),
    time: c.finish > 0 ? lapText(c.finish - COUNTDOWN) : say(world, c.out ? "results.dnfShort" : "results.running"),
    gap: c.finish > 0 && c !== winner && winner.finish > 0 ? `+${(ms(c.finish - winner.finish) / 1000).toFixed(3)}` : "",
    points: c.finish > 0 ? POINTS[c.place - 1] ?? 0 : 0,
    player: c.seat >= 0 && mine.includes(c),
  }));
  const lap = mine.reduce((m, c) => (c.bestLap > 0 && (m === 0 || c.bestLap < m) ? c.bestLap : m), 0);
  return {
    title: say(world, title, { n: (best?.seat ?? 0) + 1 }),
    place: best?.place ?? FIELD,
    field: FIELD,
    fieldText: say(world, "results.field", { place: best?.place ?? FIELD, field: FIELD }),
    bestLap: lap > 0 ? lapText(lap) + (record.newBest ? " ★" : "") : "—",
    points: best !== null && best.finish > 0 ? POINTS[best.place - 1] ?? 0 : 0,
    rows,
    rewards: [{ name: say(world, "results.wins"), amount: record.wins }, { name: say(world, "results.races"), amount: record.races }],
  };
}

/**
 * A screen document up while `shown`: og.ui.screen pushes it the first tick
 * and builds it with its viewmodel each tick; the tick it is not shown it is
 * popped. Answers the events it emitted (its buttons' `emit`, Back's).
 */
function layer(world: World, name: string, shown: boolean, vm: object = {}): string[] {
  if (!shown) {
    if (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

/**
 * This device's platform answers (the settings loaded, the record read and
 * written): a system that writes no shared state, so it hears them in a
 * room too (an answer is this peer's alone). List it before Ui.
 */
export const Device = defineSystem("Device", { reads: [og.Lockstep], writes: [og.Audio] }, (_q, _time, world) => platform(world));

/** The screens, the race's start and results, the hold, the buses. */
export const Ui = defineSystem("Ui", { reads: [og.Gamepad, og.Pointers, og.PlayerInput, og.Lockstep], writes: [Game, Car, og.Transform, og.Audio] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  if (time.tick === 0) applyPrefs(world);
  choices(world, g);
  // The settings hold this device's time alone, so a room's settings hold nothing.
  const hold = g.phase === Phase.Race && (g.paused || (screens.settingsOpen && !inRoom(world)));
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  finishRace(world, g);

  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Race ? "race" : "results";
  const shown = og.ui.screenTransition("Screens", screen, { kind: prefs.reducedMotion ? "fade" : "dissolve", durationMs: 500 }).shown;
  for (const e of layer(world, "main-menu", shown === "title", { touch: usesTouch() })) {
    if (!screens.settingsOpen && e === "play" && g.phase === Phase.Title) press("throttle");
    if (e === "settings") screens.settingsOpen = true;
  }
  const fromSettings = og.ui.memory("pause.fromSettings", 0);
  const pauseUp = shown === "race" && g.paused && !screens.settingsOpen;
  for (const e of layer(world, "pause", pauseUp)) {
    if (e === "resume") press("pause");
    if (e === "settings") {
      screens.settingsOpen = true;
      og.ui.remember("pause.fromSettings", 1);
    }
    if (e === "quit") press("menu");
  }
  if (pauseUp && fromSettings === 1) {
    og.ui.setFocus("pause/Pause/Panel/Menu/Settings");
    og.ui.remember("pause.fromSettings", 0);
  }
  for (const e of layer(world, "race-results", shown === "results" && g.phase === Phase.Results, g.phase === Phase.Results ? resultsModel(world, g) : {})) {
    if (e === "restart") press("throttle");
    if (e === "continue") press("menu");
  }
  if (shown === "title" || shown === "results") og.ui.setNavigation(true);
  else if (shown === "race" && !g.paused) hud(world, g);
  // The touch controls (drawn by Seats next tick) and their taps on the ui bus. A
  // tap is this device's own: a room plays none, which would part the peers.
  touchHud.shown = shown === "race" && g.phase === Phase.Race && !g.paused;
  touchHud.world = world;
  for (let seat = 0; seat < SEATS; seat++) touchHud.boost[seat] = must(world.get(CARS[seat]!, Car)).boost;
  if (touchState.taps > 0 && !ONLINE) g.sfx |= Sound.Tap;
  touchState.taps = 0;
  settings(world);
});

