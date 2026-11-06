// Holdout's screens on og.ui and the scifi style (assets/styles/scifi): the
// title over the arena from the trooper's spawn and the pause menu (the game's
// screen documents, assets/ui/main-menu.ogt and pause.ogt), the HUD (the
// shooter kit's crosshair and hit markers, ammunition, killfeed, damage
// arcs and low-health edge; health bottom-left; the wave, foes left and
// the score at the top; Pause top-right), the settings dialog and the
// run's summary. Two seats on one device each get their own HUD in their
// own half. Screens change through a dissolve (a fade with reduced
// motion). Every string is a key of assets/strings/*.ogt.
//
// Every choice that changes the world reaches it through a seat's input
// (src/seats.ts press), so a room makes it on one tick on every peer: on
// the title Hold the line presses Jump and Duel presses Reload; Again
// presses Jump; Title and Leave the arena press Reload (no menu control is
// among the starter's buttons); Pause and Resume press Pause. The touch HUD
// (a stick, a drag to look, Fire, Jump and Reload) is drawn for each local
// seat by src/seats.ts's Seats (useTouchControls).
//
// The Ui system also starts and ends runs, holds the world while paused,
// sets the buses and keeps the settings (this device's, src/device.ts) and
// the record (`{runs, wins, best}`) in og.platform; a room keeps none.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { shooter } from "@origingame/engine/extensions/shooter/index.ts";
import { DUEL_FRAGS, Ending, Game, Hero, Mode, Phase, Sound, Tuning, WAVES, WaveState } from "./components.ts";
import { prefs } from "./device.ts";
import { beginWaves, waveSize } from "./foes.ts";
import { FEET, heroOf, ISO_YAW, lazyTuning, resetHero, spawnOf } from "./hero.ts";
import { bit, LOCAL, localSeats, ONLINE, press, pressedNow, SEATS, setLookLive, useTouchControls, usesTouch } from "./seats.ts";
import { FIRST_PERSON, ISOMETRIC, SeatLook } from "./view.ts";
import { clearMarks, must, point } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** Ticks between a run's end and its summary: the moment plays out first. */
export const OVER_DELAY = 90;
/** Ticks from a run's end until Again and Menu answer on the summary. */
const SUMMARY_TICKS = OVER_DELAY + 20;

/** The tokens' spacing scale in CSS px. */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16 } as const;
const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

type GameV = Value<typeof Game>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

/** Whether a room runs: no record is kept and no screen holds the world for one peer. */
function inRoom(world: World): boolean {
  if (!ONLINE) return false;
  const lock = world.resource(og.Lockstep);
  return lock.ok && lock.value.running;
}

/** A new run of a mode: the troopers at their starts, whole, the arena clear. */
export function startRun(world: World, g: GameV, mode: number): void {
  const t = lazyTuning(world);
  g.phase = Phase.Play;
  g.mode = SEATS > 1 ? mode : Mode.Holdout;
  g.ending = Ending.None;
  g.winner = -1;
  g.paused = false;
  g.time = 0;
  g.score = 0;
  g.kills = 0;
  g.headshots = 0;
  g.shots = 0;
  g.hits = 0;
  g.downs = 0;
  g.alive = 0;
  g.overTicks = 0;
  g.newBest = false;
  g.start = true;
  beginWaves(g);
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    const me = must(world.get(hero, Hero));
    const duel = g.mode === Mode.Duel;
    const at = duel ? point(world, `duel-${seat}`) : spawnOf(world, seat);
    resetHero(world, hero, me, t, duel ? { x: at.x, y: at.y + FEET + 0.05, z: at.z } : at);
    Object.assign(me, { kills: 0, frags: 0, score: 0, jumps: 0, hitTick: -1000, hurtTick: -1000 });
    // The duellists start facing each other across the arena.
    if (duel && !ISOMETRIC) must(world.set(hero, SeatLook, { yaw: seat === 0 ? 0 : Math.PI }));
    must(world.set(hero, Hero, me));
  }
  clearMarks(world);
  if (!inRoom(world)) {
    g.runs += 1;
    saveRecord(world, g);
  }
}

/** Back to the title: the arena cleared behind it. */
export function toTitle(g: GameV): void {
  g.phase = Phase.Title;
  g.paused = false;
  g.ending = Ending.None;
  g.start = true;
  g.wave = 0;
  g.alive = 0;
}

/** The record ({runs, wins, best}), written at the revision last read or written; one call at a time. */
function saveRecord(world: World, g: GameV): void {
  if (g.recordRequest !== 0 || g.recordWrite !== 0) {
    g.recordDirty = true;
    return;
  }
  g.recordDirty = false;
  const r = og.platform.storeSet(world, { key: "record", value: { runs: g.runs, wins: g.wins, best: g.best }, revision: g.recordRevision });
  if (!r.ok) return;
  g.recordWrite = r.value;
  g.recordSentRuns = g.runs;
  g.recordSentWins = g.wins;
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

export function setLanguage(world: World, index: number): void {
  prefs.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[prefs.language]!.code);
}

function prefsValue() {
  return { masterVolume: prefs.master, musicVolume: prefs.music, sfxVolume: prefs.effects, language: LANGUAGES[prefs.language]!.code, reducedMotion: prefs.reducedMotion, lookSensitivity: prefs.sensitivity };
}

/**
 * og.platform: the settings and the record asked for on tick 0 and applied
 * when they answer; a record write refused as stale is read again, merged
 * (the stored counts plus what this device added, the better best) and
 * written at the new revision. In a room the record is left alone.
 */
function platform(world: World, g: GameV): void {
  if (world.tick === 0) {
    prefs.loadRequest = og.platform.loadSettings(world);
    if (!ONLINE) {
      const r = og.platform.storeGet(world, "record");
      g.recordRequest = r.ok ? r.value : 0;
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
        if (typeof s.lookSensitivity === "number") prefs.sensitivity = s.lookSensitivity;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, at);
        applyPrefs(world);
      }
    }
  }
  if (g.recordRequest !== 0) {
    const r = og.platform.result(world, g.recordRequest);
    if (r !== null) {
      g.recordRequest = 0;
      const d = (r.ok ? r.data : null) as { value?: { runs?: number; wins?: number; best?: number } | null; revision?: number } | null;
      const runs = d?.value?.runs ?? 0;
      const wins = d?.value?.wins ?? 0;
      g.runs = runs + g.runs - g.recordRuns;
      g.wins = wins + g.wins - g.recordWins;
      g.best = Math.max(g.best, d?.value?.best ?? 0);
      g.recordRuns = runs;
      g.recordWins = wins;
      if (typeof d?.revision === "number") g.recordRevision = d.revision;
      if (g.recordMerge || g.recordDirty) {
        g.recordMerge = false;
        saveRecord(world, g);
      }
    }
  }
  if (g.recordWrite !== 0) {
    const r = og.platform.result(world, g.recordWrite);
    if (r !== null) {
      g.recordWrite = 0;
      if (r.ok) {
        g.recordRevision = (r.data as { revision: number }).revision;
        g.recordRuns = g.recordSentRuns;
        g.recordWins = g.recordSentWins;
        if (g.recordDirty) saveRecord(world, g);
      } else if (r.error?.code === "conflict") {
        g.recordMerge = true;
        const again = og.platform.storeGet(world, "record");
        g.recordRequest = again.ok ? again.value : 0;
      } else if (g.recordDirty) saveRecord(world, g);
    }
  }
}

/** The run's clock and its end: the summary a beat after the arena is held, overrun or the duel won. */
function clock(world: World, g: GameV, dt: number): void {
  if (g.phase === Phase.Over) g.overTicks += 1;
  if (g.phase !== Phase.Play) return;
  if (g.ending === Ending.None) {
    g.time += dt;
    return;
  }
  g.overTicks += 1;
  if (g.overTicks !== OVER_DELAY) return;
  g.phase = Phase.Over;
  if (inRoom(world)) return;
  if (g.mode === Mode.Holdout) {
    if (g.ending === Ending.Held) g.wins += 1;
    g.newBest = g.score > g.best;
    if (g.newBest) g.best = g.score;
  }
  saveRecord(world, g);
  must(og.platform.telemetry(world, "run.over", { mode: g.mode === Mode.Duel ? "duel" : "holdout", ending: ["none", "held", "overrun", "duel"][g.ending]!, wave: g.wave, score: g.score, time: Math.round(g.time) }));
}

/** A seat's control went down this tick (against Hero.before, which Weapon moves on after Ui). */
function pressedBy(world: World, g: GameV, name: string): boolean {
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    if (pressedNow(must(world.get(hero, og.PlayerInput)).buttons, must(world.get(hero, Hero)).before, name)) return true;
  }
  return false;
}

/** This device's own screen state: the settings dialog. */
let settingsOpen = false;

/**
 * The seats' choices, from their input: Pause opens and closes the pause
 * menu; on the title Jump holds the line and (two seats) Reload starts the
 * duel; paused, Reload leaves the arena; on the summary Jump goes again and
 * Reload goes back to the title (the menus' buttons press the same controls,
 * so a room's peers leave together).
 */
function choices(world: World, g: GameV): void {
  if (pressedBy(world, g, "pause")) {
    if (g.paused) g.paused = false;
    else if (g.phase === Phase.Play && g.ending === Ending.None) g.paused = true;
  }
  if (g.phase === Phase.Title) {
    if (pressedBy(world, g, "jump")) startRun(world, g, Mode.Holdout);
    else if (SEATS > 1 && pressedBy(world, g, "reload")) startRun(world, g, Mode.Duel);
  } else if (g.phase === Phase.Play) {
    if (g.paused && pressedBy(world, g, "reload")) toTitle(g);
  } else if (g.overTicks >= SUMMARY_TICKS) {
    if (pressedBy(world, g, "jump")) startRun(world, g, g.mode);
    else if (pressedBy(world, g, "reload")) toTitle(g);
  }
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.sm, cross: "center", padding: { left: S.md, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

/** The run's line at the top: the wave and the foes left (holdout), or the frags (duel), and the score. */
function topLine(world: World, g: GameV): void {
  og.ui.anchor("Run", { anchor: "top", gap: S.sm, cross: "center" }, () => {
    if (g.mode === Mode.Duel) {
      capsule("Duel", () => {
        for (let seat = 0; seat < SEATS; seat++) {
          const me = must(world.get(heroOf(g, seat), Hero));
          if (seat > 0) og.ui.label("Dash", "—", { color: { token: "inkFaint" } });
          og.ui.label(`P${seat}`, say(world, "hud.player", { n: seat + 1 }), { step: "sm", color: { token: seat === 0 ? "warn" : "info" }, nowrap: true });
          og.ui.label(`Frags${seat}`, `${me.frags}`, { step: "lg", font: "mono", nowrap: true });
        }
        og.ui.label("To", say(world, "hud.to", { n: DUEL_FRAGS }), { step: "sm", color: { token: "inkMuted" }, nowrap: true });
      });
      return;
    }
    capsule("Wave", () => {
      og.ui.label("Label", say(world, "hud.wave", { n: Math.max(1, g.wave), total: WAVES }), { font: "display", nowrap: true });
      const left = g.waveState === WaveState.Fighting ? g.alive + g.toSpawn : waveSize(Math.max(1, g.wave));
      og.ui.label("Left", say(world, "hud.left", { n: left }), { font: "mono", color: { token: "inkMuted" }, nowrap: true });
    });
    capsule("Score", () => og.ui.label("Value", `${g.score}`, { step: "lg", font: "mono", nowrap: true }));
  });
  if (g.waveState === WaveState.Breather && g.mode === Mode.Holdout && g.ending === Ending.None) {
    og.ui.anchor("Incoming", { anchor: "center", offsetY: -110, direction: "column", cross: "center", gap: S.xs }, () => {
      og.ui.label("Title", say(world, g.wave === 1 ? "hud.first" : "hud.incoming", { n: g.wave }), { step: "displaySm", font: "display", nowrap: true });
      og.ui.label("Count", `${Math.ceil(Math.max(0, g.waveTimer))}`, { step: "xl", font: "mono", color: { token: "warn" }, nowrap: true });
    });
  }
}

/** One seat's own HUD: its crosshair, ammunition, health, the damage arcs and the low-health edge. */
function seatHud(world: World, g: GameV, seat: number, half: -1 | 0 | 1): void {
  const t = lazyTuning(world);
  const hero = heroOf(g, seat);
  const me = must(world.get(hero, Hero));
  const vp = og.ui.frameViewport();
  const ui = og.ui.tick();
  const ago = (tick: number) => ui - (world.tick - tick);
  const offsetX = (half * vp.width) / 4;
  const suffix = half === 0 ? "" : `${seat}`;
  const touch = usesTouch();
  if (me.down) {
    og.ui.anchor(`Down${suffix}`, { anchor: "center", offsetX, direction: "column", cross: "center", gap: S.xs }, () => {
      og.ui.label("Title", say(world, "hud.down"), { step: "xl", font: "display", color: { token: "danger" }, nowrap: true });
      og.ui.label("Hint", say(world, g.mode === Mode.Duel ? "hud.back.duel" : "hud.back.wave"), { color: { token: "inkMuted" }, nowrap: true });
    });
    return;
  }
  // The rounds' spread on screen: the cone's edge in CSS px at the view's field of view.
  const fov = FIRST_PERSON ? 75 : 55;
  const spreadPx = ((t.spread + me.bloom) / fov) * vp.height;
  const hitAt = world.tick - me.hitTick < 40 ? ago(me.hitTick) : null;
  const kind = (["hit", "headshot", "kill"] as const)[me.hitKind]!;
  const aim = { style: "cross", spread: spreadPx, gap: 5, length: 9, thickness: 2, dot: true, hitAt, hit: kind } as const;
  if (!ISOMETRIC && half === 0) shooter.crosshair(`Aim${suffix}`, aim);
  else if (!ISOMETRIC) {
    // Two seats here: each half's crosshair, hit markers and all, built in place in a group at the half's centre.
    og.ui.anchor(`Aim${suffix}`, { anchor: "center", offsetX, direction: "column", cross: "center" }, () => shooter.crosshair("Reticle", { ...aim, anchor: null }));
  }
  const hits = world.tick - me.hurtTick < 90 ? [{ angle: hurtAngle(world, hero, me), at: ago(me.hurtTick), strength: 0.8 }] : [];
  if (half === 0) {
    shooter.damageDirection(`Hurt${suffix}`, { hits });
    shooter.lowHealthVignette(`Low${suffix}`, { health: me.health / t.health, hitAt: world.tick - me.hurtTick < 30 ? ago(me.hurtTick) : null });
  }
  // On a touch screen the bottom is the controls': the health and the rounds
  // stack at the top, under the run's line, clear of the crosshair.
  const corner = touch ? (half === 1 ? "topRight" : "topLeft") : half === 1 ? "bottomRight" : "bottomLeft";
  const inColumn = LOCAL || touch;
  og.ui.anchor(`Vitals${suffix}`, { anchor: corner, offsetY: touch ? 60 : 0, direction: "column", gap: S.sm, cross: half === 1 ? "end" : "start" }, () => {
    if (LOCAL) og.ui.label("Seat", say(world, "hud.player", { n: seat + 1 }), { step: "sm", font: "display", color: { token: seat === 0 ? "warn" : "info" }, nowrap: true });
    capsule("Health", () => {
      og.ui.icon("Glyph", "heart", { size: 20, tint: { token: me.health / t.health < 0.35 ? "danger" : "ink" } });
      og.ui.meter("Bar", me.health, t.health, { kind: "health", width: px(touch ? (LOCAL ? 64 : 110) : 160) });
      og.ui.label("Value", `${Math.ceil(me.health)}`, { font: "mono", nowrap: true, width: px(34), align: "end" });
    });
    if (inColumn) ammo(world, me, t, null);
  });
  if (!inColumn) ammo(world, me, t, "bottomRight");
}

function ammo(world: World, me: Value<typeof Hero>, t: Value<typeof Tuning>, anchor: "bottomRight" | null): void {
  // Two seats on a phone: each half is under 200 px wide, so the rounds alone (no name, mode, pips or reload words; the ring stays).
  const narrow = LOCAL && usesTouch();
  shooter.ammoCounter("Ammo", {
    magazine: me.mag, magazineMax: t.magazine, reserve: me.reserve, weapon: narrow ? "" : say(world, "hud.rifle"), fireMode: narrow ? "" : say(world, "hud.auto"),
    reloading: me.reload > 0 ? 1 - me.reload / t.reloadSeconds : null, reloadLabel: narrow ? "" : say(world, "hud.reloading"), reloadHint: narrow ? "" : say(world, usesTouch() ? "hud.reload.touch" : "hud.reload.keys"),
    anchor, pips: !narrow,
  });
}

/** Where a seat's last hurt came from, clockwise from where it looks. */
function hurtAngle(world: World, hero: ReturnType<typeof heroOf>, me: Value<typeof Hero>): number {
  const p = must(world.get(hero, og.Transform)).position;
  const look = must(world.get(hero, SeatLook));
  // Isometric: from the screen's up (the view's fixed heading), not the trooper's facing.
  const yaw = ISOMETRIC ? ISO_YAW : look.yaw;
  const to = Math.atan2(-(me.hurtX - p.x), -(me.hurtZ - p.z)); // lockstep-ok: the HUD's arc, drawn on this device
  let a = yaw - to;
  while (a > Math.PI) a -= 2 * Math.PI;
  while (a < -Math.PI) a += 2 * Math.PI;
  return a;
}

/** The killfeed: the last downed foes and frags, newest first. */
const feed: { key: string; killer: string; victim: string; local: boolean; at: number; headshot: boolean; killerTeam: number; victimTeam: number }[] = [];
const feedKills = [0, 0];
const feedFrags = [0, 0];

function killfeed(world: World, g: GameV): void {
  const ui = og.ui.tick();
  for (let seat = 0; seat < SEATS; seat++) {
    const me = must(world.get(heroOf(g, seat), Hero));
    const name = say(world, "hud.player", { n: seat + 1 });
    const mine = localSeats(world).includes(seat);
    if (me.kills > feedKills[seat]!) feed.push({ key: `k${seat}-${me.kills}`, killer: name, victim: say(world, "feed.foe"), local: mine, at: ui, headshot: false, killerTeam: seat, victimTeam: 2 });
    if (me.frags > feedFrags[seat]!) feed.push({ key: `f${seat}-${me.frags}`, killer: name, victim: say(world, "hud.player", { n: 2 - seat }), local: mine, at: ui, headshot: false, killerTeam: seat, victimTeam: 1 - seat });
    feedKills[seat] = me.kills;
    feedFrags[seat] = me.frags;
  }
  while (feed.length > 8) feed.shift();
  shooter.killfeed("Feed", { entries: feed, max: LOCAL ? 3 : 4, durationMs: 4000 });
}

function hud(world: World, g: GameV): void {
  og.ui.setNavigation(false);
  topLine(world, g);
  if (g.ending === Ending.None) {
    if (LOCAL) for (let seat = 0; seat < SEATS; seat++) seatHud(world, g, seat, seat === 0 ? -1 : 1);
    else seatHud(world, g, localSeats(world)[0] ?? 0, 0);
  }
  killfeed(world, g);
  og.ui.anchor("Corner", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    // Pause acts through the seat's input (src/seats.ts), so a room pauses on one tick.
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) press("pause");
  });
}

/** The touch HUD's state on this device: each seat's look finger and what answered on the ui bus. */
const touchState = { shown: false, lookId: [-1, -1], lookAt: [{ x: 0, y: 0 }, { x: 0, y: 0 }], taps: 0, held: [0, 0], world: null as World | null };
/** Degrees the view turns per CSS px a finger drags (times the settings' sensitivity). */
const LOOK_DEG_PER_PX = 0.32;

/**
 * Holdout's touch HUD for a seat, drawn by Seats while touch shows and a
 * run is on screen: the stick in its bottom corner, Fire (big), Jump and
 * Reload beside it, and a finger that lands elsewhere in its part of the
 * screen (the right half alone, its own half with two) turning the view as
 * it drags until it lifts. Fire held fires.
 */
function touchControls(seat: number): { x: number; y: number; lx: number; ly: number; buttons: number } {
  const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
  const world = touchState.world;
  if (!touchState.shown || world === null) {
    touchState.lookId[seat] = -1;
    return out;
  }
  const vp = og.ui.frameViewport();
  const right = LOCAL && seat === 1;
  const stick = () => {
    const s = og.ui.joystick("Stick", { size: LOCAL ? 96 : 140, deadZone: 0.12 });
    const was = touchState.held[seat]!;
    if (s.active && was === 0) touchState.taps += 1;
    touchState.held[seat] = s.active ? 1 : 0;
    if (s.active) {
      out.x = s.x;
      out.y = -s.y;
    }
  };
  // Reload and Jump stacked by Fire (alone: one cluster in the bottom right
  // corner; two seats: a column over each seat's stick, inside its half).
  const hands = () => {
    og.ui.column("Small", { gap: S.sm, cross: "center" }, () => {
      if (og.ui.actionButton("Reload", say(world, "hud.reload"), { size: LOCAL ? 46 : 58 }).clicked) {
        out.buttons |= bit("reload");
        touchState.taps += 1;
      }
      if (og.ui.actionButton("Jump", say(world, "hud.jump"), { size: LOCAL ? 46 : 58 }).clicked) {
        out.buttons |= bit("jump");
        touchState.taps += 1;
      }
    });
    if (og.ui.actionButton("Fire", say(world, "hud.fire"), { icon: "bolt", size: LOCAL ? 58 : 80, primary: true }).pressed) out.buttons |= bit("fire");
  };
  if (LOCAL) {
    // Each seat's half of a phone is narrow: its buttons stack over its stick.
    og.ui.anchor(`TouchMove${seat}`, { anchor: right ? "bottomRight" : "bottomLeft", direction: "column", gap: S.sm, cross: "center" }, () => {
      og.ui.column("Hands", { gap: S.sm, cross: "center" }, hands);
      stick();
    });
  } else {
    og.ui.anchor(`TouchMove${seat}`, { anchor: "bottomLeft" }, stick);
    og.ui.anchor(`TouchHands${seat}`, { anchor: "bottomRight", gap: S.md, cross: "end" }, hands);
  }
  // The look: a finger landing in this seat's part of the screen, clear of the controls, turns the view as it drags.
  const ptr = must(world.resource(og.Pointers));
  let id = touchState.lookId[seat]!;
  if (id >= 0 && !ptr.down(id)) id = -1;
  const k = LOOK_DEG_PER_PX * prefs.sensitivity * (ISOMETRIC ? 1.6 : 1);
  if (id >= 0) {
    const p = ptr.position(id);
    const last = touchState.lookAt[seat]!;
    out.lx = (p.x - last.x) * k;
    out.ly = -(p.y - last.y) * k;
    touchState.lookAt[seat] = { x: p.x, y: p.y };
  } else {
    const x0 = LOCAL ? (seat === 0 ? 0 : vp.width / 2) : vp.width * 0.42;
    const x1 = LOCAL ? (seat === 0 ? vp.width / 2 : vp.width) : vp.width;
    const top = vp.safeArea.top + 70;
    const bottom = vp.height - vp.safeArea.bottom - (LOCAL ? 380 : 200);
    for (let n = 0; n < 5; n++) {
      if (!ptr.pressed(n) || touchState.lookId.includes(n)) continue;
      const at = ptr.start(n);
      if (at.x < x0 || at.x >= x1 || at.y < top || at.y > bottom) continue;
      id = n;
      const p = ptr.position(n);
      touchState.lookAt[seat] = { x: p.x, y: p.y };
      break;
    }
  }
  touchState.lookId[seat] = id;
  return out;
}
useTouchControls(touchControls);

function settings(world: World): void {
  let done = false;
  const before = JSON.stringify(prefsValue());
  const m = og.ui.modal("Settings", settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(440, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number, max = 1): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(120), nowrap: true });
        out = og.ui.slider("Slider", value / max, { step: 0.05 }).value * max;
        og.ui.label("Value", `${Math.round((out / max) * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    prefs.master = slider("Master", "settings.master", prefs.master);
    prefs.music = slider("Music", "settings.music", prefs.music);
    prefs.effects = slider("Effects", "settings.effects", prefs.effects);
    prefs.sensitivity = Math.max(0.2, slider("Look", "settings.look", prefs.sensitivity, 2));
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), prefs.language, { horizontal: true });
    if (r.selected !== prefs.language) setLanguage(world, r.selected);
    prefs.reducedMotion = og.ui.toggle("Motion", prefs.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue()) !== before) applyPrefs(world);
  if (settingsOpen && (done || m.dismissed)) {
    settingsOpen = false;
    must(og.platform.saveSettings(world, prefsValue()));
    prefs.saves += 1;
  }
}

function seconds(s: number): string {
  const m = Math.floor(s / 60);
  const r = Math.floor(s % 60);
  return `${m}:${r < 10 ? "0" : ""}${r}`;
}

/** The run's summary: how it ended, the wave, the score, the accuracy and the record (a duel: the frags). */
function summary(world: World, g: GameV): "again" | "menu" | null {
  let chose: "again" | "menu" | null = null;
  const duel = g.mode === Mode.Duel;
  const won = g.ending === Ending.Held || duel;
  const title = duel ? "over.duel" : g.ending === Ending.Held ? "over.held" : "over.overrun";
  og.ui.modal("Summary", g.phase === Phase.Over, { title: say(world, title, { n: g.winner + 1 }), locked: true, width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => {
      og.ui.transition("Icon", true, { kind: "pop" }, () => og.ui.icon("Glyph", won ? "trophy" : "skull", { size: 48, tint: { token: won ? "primary" : "danger" } }));
      if (won && g.overTicks >= OVER_DELAY + 20 && !prefs.reducedMotion) og.ui.burst({ count: 14 });
    });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      const accuracy = g.shots > 0 ? `${Math.round((100 * g.hits) / g.shots)}%` : "—";
      if (duel) {
        for (let seat = 0; seat < SEATS; seat++) {
          const me = must(world.get(heroOf(g, seat), Hero));
          og.ui.keyValue(`P${seat}`, say(world, "hud.player", { n: seat + 1 }), say(world, "over.frags", { n: me.frags }), true);
        }
      } else {
        og.ui.keyValue("Wave", say(world, "over.wave"), `${Math.min(g.wave, WAVES)}/${WAVES}`, true);
        og.ui.keyValue("Score", say(world, "over.score"), `${g.score}${g.newBest ? " ★" : ""}`, true);
        og.ui.keyValue("Kills", say(world, "over.kills"), `${g.kills}`, true);
      }
      og.ui.keyValue("Accuracy", say(world, "over.accuracy"), accuracy, true);
      og.ui.keyValue("Time", say(world, "over.time"), seconds(g.time), duel);
      if (!duel) og.ui.keyValue("Best", say(world, "over.best"), g.best > 0 ? `${g.best}` : say(world, "over.none"), false);
    });
    if (!duel && !inRoom(world)) og.ui.label("Record", say(world, "over.record", { runs: g.runs, wins: g.wins }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm }, () => {
      if (og.ui.button("Again", say(world, "over.again"), { variant: "primary", width: grow }).clicked) chose = "again";
      if (og.ui.button("Menu", say(world, "over.menu"), { width: grow }).clicked) chose = "menu";
    });
  });
  return chose;
}

/** A screen document up while `shown`; answers the events it emitted. */
function layer(world: World, name: string, shown: boolean, vm: object = {}): string[] {
  if (!shown) {
    if (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

/** The screens, the run's start and end, the hold, the buses, the platform's answers. */
export const Ui = defineSystem("Ui", { reads: [Tuning, og.Pointers, og.PlayerInput, og.Lockstep, Hero, SeatLook], writes: [Game, Hero, SeatLook, og.Audio, og.Transform, og.CharacterController, og.Decal] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  platform(world, g);
  if (time.tick === 0) applyPrefs(world);
  choices(world, g);
  // Paused holds the world; a room never holds for one peer's pause menu alone (Pause is a seat's input, so it pauses every peer).
  const hold = g.phase === Phase.Play && g.paused;
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  clock(world, g, time.scaledDt);

  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "over";
  const shown = og.ui.screenTransition("Screens", screen, { kind: prefs.reducedMotion ? "fade" : "dissolve", durationMs: 500 }).shown;
  const touch = usesTouch();
  for (const e of layer(world, "main-menu", shown === "title", { touch, duel: SEATS > 1, online: ONLINE })) {
    if (settingsOpen) continue;
    if (e === "play" && g.phase === Phase.Title) press("jump");
    if (e === "duel" && g.phase === Phase.Title) press("reload");
    if (e === "settings") settingsOpen = true;
  }
  const pauseUp = shown === "play" && g.paused && !settingsOpen;
  for (const e of layer(world, "pause", pauseUp)) {
    if (e === "resume") press("pause");
    if (e === "settings") settingsOpen = true;
    if (e === "quit") press("reload");
  }
  if (shown === "title" || shown === "over") og.ui.setNavigation(true);
  if (shown === "play") hud(world, g);
  else if (shown === "over") {
    const c = summary(world, g);
    if (c === "again") press("jump");
    if (c === "menu") press("reload");
  }
  // The touch HUD (drawn by Seats next tick) and its taps on the ui bus; a room plays none (they would part the peers).
  touchState.shown = shown === "play" && !g.paused && g.ending === Ending.None;
  touchState.world = world;
  if (touchState.taps > 0 && !ONLINE) g.sfx |= Sound.Tap;
  touchState.taps = 0;
  // The mouse turns the view only in a run: on the screens the pointer is the player's.
  setLookLive(g.phase === Phase.Play && !g.paused && !settingsOpen && g.ending === Ending.None);
  settings(world);
});

/** The language as the facts report it. */
export function languageCode(): string {
  return LANGUAGES[prefs.language]!.code;
}

/** Opens the settings as their button does (the playtests). */
export function openSettings(): void {
  settingsOpen = true;
}
export function settingsShown(): boolean {
  return settingsOpen;
}
