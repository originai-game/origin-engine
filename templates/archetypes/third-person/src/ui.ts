// Ember Vale's screens on og.ui and the style in assets/styles/ember (the
// engine's fantasy pack): the title over an orbit of the vale and the
// pause menu (the game's screen documents, assets/ui/main-menu.ogt and
// pause.ogt), the HUD (hearts and beacons top-left, the dusk clock and
// pause top-right, the kindling prompt and its progress at the bottom; on
// a touch screen a joystick, Jump and Kindle, and a drag on the right half
// turning the view), the settings dialog and the run's summary. Screens change through a dissolve (a fade with
// reduced motion). Every string is a key of assets/strings/*.ogt.
//
// Every choice that changes the world reaches it through a seat's input
// (src/seats.ts press), so a room makes it on one tick on every peer: Set
// out and Again press Jump, Menu and Leave the vale press Menu (a control with
// no key of its own), Pause and
// Resume press Pause. The touch HUD is drawn for seat 0 by src/seats.ts's
// Seats (useTouchControls), and gives its stick, buttons and look as that
// seat's input.
//
// The Ui system also starts runs, moves the clock (og.DayNight, held by
// the game so a pause holds it), ends runs (every beacon lit, the hearts
// gone, or night), holds the world while paused, sets the buses and keeps
// the settings and the record in og.platform (the record written at the
// revision read, and merged when another device wrote it first).

import { og, defineSystem } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { Beacon, BEACONS, Ending, Game, HEARTS, Hero, Phase, Prefs, Sound, Tuning } from "./components.ts";
import { heroOf, hypot, lazyTuning } from "./hero.ts";
import { bit, LOCAL, localSeats, ONLINE, press, pressedNow, SEATS, setLookLive, useTouchControls } from "./seats.ts";
import { HOURS, must, setHour } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** Ticks between a run's end and its summary: the moment plays out first. */
export const OVER_DELAY = 90;

/** The tokens' spacing scale in CSS px. */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16 } as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** A new run: the hero at the start, hearts full, the beacons dark, the clock at mid-afternoon. */
export function startRun(world: World, g: GameV): void {
  g.phase = Phase.Play;
  g.ending = Ending.None;
  g.paused = false;
  g.hearts = HEARTS;
  g.hurts = 0;
  g.lit = 0;
  g.litMask = 0;
  g.time = 0;
  g.hour = HOURS.start;
  g.kindle = 0;
  g.near = -1;
  g.jumps = 0;
  g.splashes = 0;
  g.grabs = 0;
  g.alarms = 0;
  g.overTicks = 0;
  g.newBest = false;
  g.start = true;
  g.runs += 1;
  setHour(world, g.hour);
  saveRecord(world, g);
}

/** Back to the title: the vale reset behind it, the beacons burning. */
export function toTitle(world: World, g: GameV): void {
  g.phase = Phase.Title;
  g.paused = false;
  g.ending = Ending.None;
  g.start = true;
  g.hour = HOURS.title;
  setHour(world, g.hour);
}

/**
 * The record ({runs, wins, best}) written at the revision last read or
 * written. One call at a time: while a read or a write is in flight the
 * newer record waits (recordDirty) and goes out when that answers, at the
 * revision the answer gives, so a late answer never stands for a newer
 * local record.
 */
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

function applyPrefs(world: World, p: PrefsV): void {
  const audio = world.resource(og.Audio);
  if (audio.ok) {
    audio.value.master = p.master;
    audio.value.music = p.music;
    audio.value.sfx = p.effects;
    audio.value.ui = p.effects;
  }
}

function setLanguage(world: World, p: PrefsV, index: number): void {
  p.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[p.language]!.code);
}

function prefsValue(p: PrefsV) {
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, language: LANGUAGES[p.language]!.code, reducedMotion: p.reducedMotion };
}

/**
 * og.platform: the settings and the record asked for on tick 0 and applied
 * when they answer; a record write refused for a stale revision (another
 * device played meanwhile) is read again, merged (the stored counts plus
 * what this device added since it last read or wrote them, the better of
 * the two best times) and written at the new revision.
 */
function platform(world: World, g: GameV, p: PrefsV): void {
  if (world.tick === 0) {
    p.loadRequest = og.platform.loadSettings(world);
    const r = og.platform.storeGet(world, "record");
    g.recordRequest = r.ok ? r.value : 0;
    return;
  }
  if (p.loadRequest !== 0) {
    const r = og.platform.result(world, p.loadRequest);
    if (r !== null) {
      p.loadRequest = 0;
      const s = ((r.ok ? r.data : null) as { value?: Record<string, unknown> } | null)?.value ?? null;
      if (s && typeof s === "object") {
        if (typeof s.masterVolume === "number") p.master = s.masterVolume;
        if (typeof s.musicVolume === "number") p.music = s.musicVolume;
        if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
        if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, p, at);
        applyPrefs(world, p);
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
      const best = d?.value?.best ?? 0;
      g.runs = runs + g.runs - g.recordRuns;
      g.wins = wins + g.wins - g.recordWins;
      if (best > 0 && (g.best === 0 || best < g.best)) g.best = best;
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
        // The store now holds what was sent, at the revision it answers.
        g.recordRevision = (r.data as { revision: number }).revision;
        g.recordRuns = g.recordSentRuns;
        g.recordWins = g.recordSentWins;
        if (g.recordDirty) saveRecord(world, g);
      } else if (r.error?.code === "conflict") {
        // Someone else wrote the record: read it again and write the merge.
        g.recordConflicts += 1;
        g.recordMerge = true;
        const again = og.platform.storeGet(world, "record");
        g.recordRequest = again.ok ? again.value : 0;
      } else if (g.recordDirty) saveRecord(world, g);
    }
  }
}

/** The clock and the endings: night at HOURS.night, the hearts gone, or (the Beacons system) every beacon lit. */
function clock(world: World, g: GameV, dt: number): void {
  const t = lazyTuning(world);
  // The summary's moments (its burst) count on from the run's end.
  if (g.phase === Phase.Over) g.overTicks += 1;
  if (g.phase !== Phase.Play) return;
  if (g.ending === Ending.None) {
    g.time += dt;
    g.hour = Math.min(HOURS.night, HOURS.start + (g.time / t.duskSeconds) * (HOURS.night - HOURS.start));
    if (g.hearts <= 0) g.ending = Ending.Caught;
    else if (g.hour >= HOURS.night) {
      g.ending = Ending.Night;
      g.sfx |= Sound.Night;
    }
  }
  setHour(world, g.hour);
  if (g.ending === Ending.None) return;
  g.overTicks += 1;
  if (g.overTicks === OVER_DELAY) {
    g.phase = Phase.Over;
    if (g.ending === Ending.Dawn) {
      g.wins += 1;
      g.newBest = g.best === 0 || g.time < g.best;
      if (g.newBest) g.best = g.time;
    }
    saveRecord(world, g);
    must(og.platform.telemetry(world, "run.over", { ending: ["none", "dawn", "night", "caught"][g.ending]!, time: Math.round(g.time), lit: g.lit, hurts: g.hurts }));
  }
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

function clockText(g: GameV): string {
  const h = Math.floor(g.hour);
  const m = Math.floor((g.hour - h) * 60);
  return `${h}:${m < 10 ? "0" : ""}${m}`;
}

function hud(world: World, g: GameV): void {
  const vp = og.ui.frameViewport();
  const touch = vp.coarsePointer;
  const t = lazyTuning(world);
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    capsule("Hearts", () => {
      for (let i = 0; i < HEARTS; i++) og.ui.icon(`Heart${i}`, "heart", { size: 22, tint: { token: i < g.hearts ? "danger" : "controlStrong" } });
    });
    capsule("Beacons", () => {
      og.ui.icon("Fire", "fire", { size: 22, tint: { token: "primary" } });
      og.ui.label("Count", say(world, "hud.beacons", { n: g.lit, total: BEACONS }), { step: "lg", font: "mono", nowrap: true });
    });
  });
  og.ui.anchor("Dusk", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    capsule("Clock", () => {
      const left = 1 - (g.hour - HOURS.start) / (HOURS.night - HOURS.start);
      og.ui.icon("Glyph", "clock", { size: 20, tint: { token: left < 0.25 ? "danger" : "ink" } });
      og.ui.label("Hour", clockText(g), { font: "mono", nowrap: true, width: px(48) });
      og.ui.meter("Left", Math.max(0, left), 1, { kind: "primary", width: px(touch ? 70 : 110) });
    });
    // Pause acts through the seat's input (src/seats.ts), so a room pauses on one tick.
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) press("pause");
  });
  // Each seat's prompt by an unlit beacon, then the kindling's progress: in
  // its own half of a split screen, or the strider this device plays.
  if (g.ending !== Ending.None) return;
  const pad = must(world.resource(og.Gamepad)).connected();
  for (const seat of LOCAL ? [0, 1] : localSeats(world)) {
    const me = must(world.get(heroOf(g, seat), Hero));
    if (me.near < 0) continue;
    const kindle = must(world.get(beaconOf(world, me.near), Beacon)).kindle;
    const offsetX = LOCAL ? (seat === 0 ? -1 : 1) * (vp.width / 4) : 0;
    og.ui.anchor(seat === 0 || !LOCAL ? "Prompt" : `Prompt${seat}`, { anchor: "bottom", offsetX, offsetY: touch ? -150 : -24, direction: "column", cross: "center", gap: S.xs }, () => {
      capsule("Hint", () => {
        og.ui.icon("Fire", "fire", { size: 18, tint: { token: "primary" } });
        og.ui.label("Text", say(world, touch && seat === 0 ? "hud.hint.touch" : pad ? "hud.hint.pad" : LOCAL && seat === 1 ? "hud.hint.keys2" : "hud.hint.keys"), { nowrap: true });
      });
      if (kindle > 0) og.ui.progress("Kindle", kindle / t.kindleSeconds, { width: px(200) });
    });
  }
}

function beaconOf(world: World, i: number): Entity {
  const e = world.find(`beacon-${i}`);
  if (e === null) throw new Error(`beacon-${i} is missing`);
  return e;
}

/** What the touch HUD needs from the world, kept by Ui each tick (it is drawn by Seats, before Ui). */
const touchHud = { shown: false, near: false, world: null as World | null };
/** The touch HUD's own state on this device: the look finger, and what answered on the ui bus. */
const touchState = { lookId: -1, lookX: 0, kindle: false, taps: 0 };

/** Radians the view turns per CSS px a finger drags across the look zone. */
const LOOK_RAD_PER_PX = 0.008;

/**
 * Ember Vale's touch HUD for seat 0, drawn by Seats while touch shows and a
 * run is on screen: the joystick bottom left (pushed all the way it runs),
 * Kindle and Jump bottom right, and a finger that lands in the look zone
 * (the right half, below the clock and above Jump and Kindle) turning the
 * view as it drags sideways, alongside the stick's thumb, until it lifts.
 * Seat 1 of a shared screen plays on keys or a pad.
 */
function touchControls(seat: number): { x: number; y: number; lx: number; ly: number; buttons: number } {
  const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
  const world = touchHud.world;
  if (seat !== 0 || !touchHud.shown || world === null) {
    touchState.lookId = -1;
    return out;
  }
  const vp = og.ui.frameViewport();
  og.ui.anchor("Move", { anchor: "bottomLeft" }, () => {
    const s = og.ui.joystick("Stick", { size: 140, deadZone: 0.12 });
    // A thumb landing on the stick answers on the ui bus too (Sound.Tap).
    const held = og.ui.memory("Held", 0);
    if (s.active && held === 0) touchState.taps += 1;
    if ((s.active ? 1 : 0) !== held) og.ui.remember("Held", s.active ? 1 : 0);
    if (s.active) {
      out.x = s.x;
      out.y = -s.y;
      if (hypot(s.x, s.y) > 0.85) out.buttons |= bit("run");
    }
  });
  og.ui.anchor("Hands", { anchor: "bottomRight", gap: S.md, cross: "end" }, () => {
    const kindle = og.ui.actionButton("Kindle", say(world, "hud.kindle"), { icon: "fire", size: 64, primary: touchHud.near }).pressed;
    // Each answers on the ui bus as it is touched (Sound.Tap), then acts.
    if (kindle && !touchState.kindle) touchState.taps += 1;
    touchState.kindle = kindle;
    if (kindle) out.buttons |= bit("kindle");
    if (og.ui.actionButton("Jump", say(world, "hud.jump"), { icon: "play", size: 72 }).clicked) {
      out.buttons |= bit("jump");
      touchState.taps += 1;
    }
  });
  // The look: degrees of the view's turn this tick (right turns it right).
  const ptr = must(world.resource(og.Pointers));
  let id = touchState.lookId;
  if (id >= 0 && !ptr.down(id)) id = -1;
  if (id >= 0) {
    const x = ptr.position(id).x;
    out.lx = ((x - touchState.lookX) * LOOK_RAD_PER_PX * 180) / Math.PI; // lockstep-ok: this device's input, sent to the room
    touchState.lookX = x;
  } else {
    const top = vp.safeArea.top + 96;
    const bottom = vp.height - vp.safeArea.bottom - 220;
    // Every pointer id (0 to 4): a finger's id is not below the count of those held.
    for (let n = 0; n < 5; n++) {
      if (!ptr.pressed(n)) continue;
      const at = ptr.start(n);
      if (at.x < vp.width / 2 || at.y < top || at.y > bottom) continue;
      id = n;
      touchState.lookX = ptr.position(n).x;
      break;
    }
  }
  touchState.lookId = id;
  return out;
}
useTouchControls(touchControls);

/** A seat's control went down this tick (against Hero.before, which the Hero system moves on after Ui). */
function pressedBy(world: World, g: GameV, name: string): boolean {
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    if (pressedNow(must(world.get(hero, og.PlayerInput)).buttons, must(world.get(hero, Hero)).before, name)) return true;
  }
  return false;
}

/** Ticks from a run's end until Jump and Kindle answer on the summary: a third of a second after it shows. */
const SUMMARY_TICKS = OVER_DELAY + 20;

/**
 * The seats' choices, from their input: Pause opens and closes the pause
 * menu (under the settings it closes them); on the title Jump sets out; paused,
 * Menu leaves the vale; on the summary Jump goes again and Menu goes back
 * to the title.
 */
function choices(world: World, g: GameV, p: PrefsV): void {
  if (g.settingsOpen) {
    if (pressedBy(world, g, "pause")) closeSettings(world, g, p);
    return;
  }
  if (pressedBy(world, g, "pause")) {
    if (g.paused) g.paused = false;
    else if (g.phase === Phase.Play && g.ending === Ending.None) g.paused = true;
  }
  if (g.phase === Phase.Title) {
    if (pressedBy(world, g, "jump")) startRun(world, g);
  } else if (g.phase === Phase.Play) {
    if (g.paused && pressedBy(world, g, "menu")) toTitle(world, g);
  } else if (g.overTicks >= SUMMARY_TICKS) {
    if (pressedBy(world, g, "jump")) startRun(world, g);
    else if (pressedBy(world, g, "menu")) toTitle(world, g);
  }
}

function settings(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  const before = JSON.stringify(prefsValue(p));
  // Locked: Back reaches the game as the seat's Pause, which closes them (choices).
  og.ui.modal("Settings", g.settingsOpen, { title: say(world, "settings.title"), locked: true, width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { step: 0.05 }).value;
        og.ui.label("Value", `${Math.round(out * 100)}`, { width: px(40), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    p.master = slider("Master", "settings.master", p.master);
    p.music = slider("Music", "settings.music", p.music);
    p.effects = slider("Effects", "settings.effects", p.effects);
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), p.language, { horizontal: true });
    if (r.selected !== p.language) setLanguage(world, p, r.selected);
    p.reducedMotion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue(p)) !== before) applyPrefs(world, p);
  if (g.settingsOpen && done) closeSettings(world, g, p);
}

/** The settings close (Done, or a seat's Pause: Escape, Start) and are saved. */
function closeSettings(world: World, g: GameV, p: PrefsV): void {
  g.settingsOpen = false;
  must(og.platform.saveSettings(world, prefsValue(p)));
  p.saves += 1;
}

function seconds(s: number): string {
  const m = Math.floor(s / 60);
  const r = Math.floor(s % 60);
  return `${m}:${r < 10 ? "0" : ""}${r}`;
}

/** The run's summary: how it ended, how long it took, what was lit and lost, the record. */
function summary(world: World, g: GameV): "again" | "menu" | null {
  let chose: "again" | "menu" | null = null;
  const won = g.ending === Ending.Dawn;
  const title = won ? "over.won" : g.ending === Ending.Caught ? "over.caught" : "over.night";
  og.ui.modal("Summary", g.phase === Phase.Over, { title: say(world, title), locked: true, width: px(Math.min(400, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center" }, () => {
      og.ui.transition("Icon", true, { kind: "pop" }, () => og.ui.icon("Glyph", won ? "fire" : g.ending === Ending.Caught ? "skull" : "clock", { size: 48, tint: { token: won ? "primary" : "inkMuted" } }));
      // From a third of a second into the summary, every frame: a burst plays only while it is called.
      if (won && g.overTicks >= OVER_DELAY + 20) og.ui.burst({ count: 14 });
    });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("Time", say(world, "over.time"), seconds(g.time), true);
      og.ui.keyValue("Beacons", say(world, "over.beacons"), `${g.lit}/${BEACONS}`, true);
      og.ui.keyValue("Hurts", say(world, "over.hurts"), `${g.hurts}`, true);
      og.ui.keyValue("Best", say(world, "over.best"), g.best > 0 ? seconds(g.best) + (g.newBest ? " ★" : "") : say(world, "over.none"), false);
    });
    og.ui.label("Record", say(world, "over.record", { runs: g.runs, wins: g.wins }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm }, () => {
      if (og.ui.button("Again", say(world, "over.again"), { variant: "primary", width: grow }).clicked) chose = "again";
      if (og.ui.button("Menu", say(world, "over.menu"), { width: grow }).clicked) chose = "menu";
    });
  });
  return chose;
}

/**
 * A screen document up while `shown`: og.ui.screen pushes it the first tick
 * and builds it with its viewmodel each tick; the tick it is not shown it is
 * popped (the top layer: the title and the pause never stack). Answers the
 * events it emitted (its buttons' `emit`, Back's).
 */
function layer(world: World, name: string, shown: boolean, vm: object = {}): string[] {
  if (!shown) {
    if (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

/** The screens, the run's start, clock and end, the hold, the buses, the platform's answers. */
export const Ui = defineSystem("Ui", { reads: [Tuning, og.Gamepad, og.Pointers, og.PlayerInput, og.Lockstep, Hero, Beacon], writes: [Game, Prefs, og.Audio, og.DayNight] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const p = must(world.resource(Prefs));
  platform(world, g, p);
  if (time.tick === 0) applyPrefs(world, p);
  choices(world, g, p);
  const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen);
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  clock(world, g, time.scaledDt);

  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "over";
  const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "dissolve", durationMs: 600 }).shown;
  // The title and the pause are the game's screen documents (assets/ui/main-menu.ogt, pause.ogt).
  for (const e of layer(world, "main-menu", shown === "title", { touch: og.ui.frameViewport().coarsePointer })) {
    if (!g.settingsOpen && e === "play" && g.phase === Phase.Title) press("jump");
    if (e === "settings") g.settingsOpen = true;
  }
  // Back from its Settings the pause keeps the focus on Settings, as the pause shipped.
  const fromSettings = og.ui.memory("pause.fromSettings", 0);
  const pauseUp = shown === "play" && g.paused && !g.settingsOpen;
  for (const e of layer(world, "pause", pauseUp)) {
    if (e === "resume") press("pause");
    if (e === "settings") {
      g.settingsOpen = true;
      og.ui.remember("pause.fromSettings", 1);
    }
    if (e === "quit") press("menu");
  }
  if (pauseUp && fromSettings === 1) {
    og.ui.setFocus("pause/Pause/Panel/Menu/Settings");
    og.ui.remember("pause.fromSettings", 0);
  }
  if (shown === "title") {
    og.ui.setNavigation(true);
  } else if (shown === "play") {
    hud(world, g);
  } else {
    og.ui.setNavigation(true);
    const c = summary(world, g);
    if (c === "again") press("jump");
    if (c === "menu") press("menu");
  }
  // The touch HUD (drawn by Seats next tick) and its taps on the ui bus. A
  // tap is this device's own: a room plays none, which would part the peers.
  touchHud.shown = shown === "play";
  touchHud.near = g.near >= 0;
  touchHud.world = world;
  if (touchState.taps > 0 && !ONLINE) g.sfx |= Sound.Tap;
  touchState.taps = 0;
  // The mouse turns the view only in a run: on the screens the pointer is the player's.
  setLookLive(g.phase === Phase.Play && !g.paused && !g.settingsOpen);
  settings(world, g, p);
  if (g.language !== p.language) g.language = p.language;
});
