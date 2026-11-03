// Meridian Loop's screens, on og.ui with the neon style (assets/styles/meridian,
// brought by the look):
//
// - the title, pause, the settings, the results and the leaderboard: the
//   game's own screen documents (assets/ui/*.ogt, ejected from the library's
//   main-menu, pause, settings, results and leaderboard with `og ui eject`):
//   Time trial, Leaderboard (the fastest laps and the ones round the player,
//   its loading, error and empty states honest), Settings (og.ui.settingsTree
//   in its slot), every word from the string tables, English and German;
// - the race: og.ui.lapTimer (the lap, the lap's time, the record, the last
//   lap and the split against the record), the three sectors as the game's
//   own rows, og.ui.minimap with the car and the ghost, og.ui.speedometer
//   (speed and gear), touch steering on a phone; the countdown, Wrong way;
// - pause (Resume, Restart, the ghost on or off, Settings, Quit), the
//   results (the race's time counting up, the session's best before it, the
//   stars by og.ui.starRating), toasts; a wipe between the screens (a fade
//   with reduced motion).
//
// The Ui system also holds time under a menu, and takes og.platform's
// answers (src/save.ts).

import { og, defineSystem } from "@origingame/engine";
import type { MapMarker, SettingEntry, SettingsPage, Texture, Value, World } from "@origingame/engine";
import { Phase, Prefs, Race, Racer, Sound, TapeChunk } from "./components.ts";
import { lapText, seconds, startRace } from "./race.ts";
import { replayAnswers, saveStart, SNAPSHOT_AT, watch } from "./replay.ts";
import { answers, readAll, readBoard } from "./save.ts";
import { uiSounds } from "./sound.ts";
import { say } from "./text.ts";
import { LAPS, mapPoint } from "./track.ts";
import { entity, must } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "de", key: "lang.de" },
] as const;

/** Ticks between the finish and the results: the car rolls on under the orbiting camera. */
export const RESULTS_DELAY = 150;

const S = { xs: 4, sm: 8, md: 12, lg: 16 } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

type RaceV = Value<typeof Race>;
type PrefsV = Value<typeof Prefs>;

export function configureUi(world: World, p: PrefsV): void {
  must(og.ui.configure(world, { reducedMotion: p.reducedMotion, contrast: p.highContrast ? "high" : "standard", sounds: uiSounds(world), toastAnchor: "bottom" }));
}

export function applyPrefs(world: World, p: PrefsV): void {
  const audio = must(world.resource(og.Audio));
  audio.master = p.muted ? 0 : p.master;
  audio.music = p.music;
  audio.sfx = p.effects;
  audio.ui = p.effects;
  configureUi(world, p);
}

export function setLanguage(world: World, p: PrefsV, index: number): void {
  p.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[p.language]!.code);
}

/** The settings as og.platform keeps them (`og.settings`). */
interface Settings {
  masterVolume?: number;
  musicVolume?: number;
  sfxVolume?: number;
  muted?: boolean;
  reducedMotion?: boolean;
  highContrast?: boolean;
  language?: string;
  ghost?: boolean;
}

/** The game's screen documents (assets/ui/), loaded once at the start. */
export const SCREENS = ["main-menu", "pause", "settings", "results", "leaderboard"] as const;

/**
 * A screen document shown while `on`: mounted (pushed when it is not on the layer stack) and built here with its
 * viewmodel, answering the events it emitted this tick (Back's too); taken off the stack when it is no longer wanted.
 */
function screen(world: World, name: (typeof SCREENS)[number], on: boolean, vm: Record<string, unknown> = {}): string[] {
  if (!on) {
    while (og.ui.layers(world).some((l) => l.screen === name)) og.ui.pop(world);
    return [];
  }
  return og.ui.screen(name, vm).events.filter((e) => e.kind === "emit" || e.kind === "back").map((e) => e.name);
}

function settingsOf(p: PrefsV): Settings {
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, muted: p.muted, reducedMotion: p.reducedMotion, highContrast: p.highContrast, language: LANGUAGES[p.language]!.code };
}

/** Settings from the platform or the settings screen, into Prefs; the ghost switch rides along. */
export function takeSettings(world: World, p: PrefsV, s: Partial<Record<string, unknown>>): void {
  if (typeof s.masterVolume === "number") p.master = s.masterVolume;
  if (typeof s.musicVolume === "number") p.music = s.musicVolume;
  if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
  if (typeof s.muted === "boolean") p.muted = s.muted;
  if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
  if (typeof s.highContrast === "boolean") p.highContrast = s.highContrast;
  if (typeof s.ghost === "boolean") p.ghost = s.ghost;
  const at = LANGUAGES.findIndex((l) => l.code === s.language);
  if (at >= 0 && at !== p.language) setLanguage(world, p, at);
  applyPrefs(world, p);
}

function saveSettings(world: World, p: PrefsV): void {
  must(og.platform.saveSettings(world, { ...settingsOf(p), ghost: p.ghost }));
  p.saves += 1;
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.sm, cross: "center", padding: { left: S.md, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

/** The lap's sectors: each one's time once crossed (ticks, 0 not yet), and the record's. */
function sectors(world: World, r: RaceV): void {
  const mine = [r.split1, r.split2 > 0 ? r.split2 - r.split1 : 0, 0];
  const recs = [r.recordS1, r.recordS2, r.recordS3];
  capsule("Sectors", () => {
    og.ui.column("Rows", { gap: 0, cross: "end" }, () => {
      mine.forEach((t, i) => {
        og.ui.row(og.ui.idi("Split", i), { gap: S.sm, cross: "center" }, () => {
          og.ui.label("Label", say(world, "hud.split", { n: i + 1 }), { step: "meta", color: { token: "inkMuted" }, nowrap: true });
          og.ui.label("Time", t === 0 ? "-:--.---" : lapText(t), { step: "meta", font: "mono", color: { token: t === 0 ? "inkFaint" : "ink" }, nowrap: true });
          if (t !== 0 && recs[i]! !== 0) {
            const d = seconds(t - recs[i]!);
            og.ui.label("Delta", `${d < 0 ? "−" : "+"}${Math.abs(d).toFixed(3)}`, { step: "meta", font: "mono", color: { token: d < 0 ? "success" : "danger" }, nowrap: true });
          }
        });
      });
    });
  });
}

function hud(world: World, r: RaceV, p: PrefsV): { pause: boolean } {
  og.ui.setNavigation(false);
  const touch = og.ui.frameViewport().coarsePointer;
  const car = must(world.get(entity(world, "car"), og.Transform)).position;
  const markers: MapMarker[] = [{ ...mapPoint(car.x, car.z), player: true }];
  if (r.ghostActive) {
    const gh = must(world.get(entity(world, "ghost"), og.Transform)).position;
    markers.push({ ...mapPoint(gh.x, gh.z), tone: "info" });
  }
  const lapTicks = r.phase === Phase.Racing ? r.clock - r.lapStart : 0;
  const done = [r.lap1, r.lap2, r.lap3].filter((t) => t > 0);
  // A time trial: no place. The lap, its time, the record, the last lap and the last split's (or the line's)
  // delta against the record, for three seconds; every caption from the tables.
  og.ui.lapTimer("Lap", {
    lap: Math.min(LAPS, r.lap),
    lapTotal: LAPS,
    currentMs: (lapTicks * 1000) / 60,
    bestMs: r.record > 0 ? (r.record * 1000) / 60 : null,
    lastMs: done.length > 0 ? (done[done.length - 1]! * 1000) / 60 : null,
    deltaMs: r.deltaShow > 0 && r.deltaAt > 0 ? seconds(r.delta) * 1000 : null,
    lapLabel: say(world, "hud.lap"),
    finalLabel: say(world, "hud.final"),
    currentLabel: say(world, "hud.current"),
    bestLabel: say(world, "hud.best"),
    lastLabel: say(world, "hud.last"),
    splitLabel: say(world, "hud.delta"),
  });
  let paused = false;
  og.ui.anchor("Side", { anchor: "topRight", direction: "column", cross: "end", gap: S.xs }, () => {
    // The Pause button, or Back (Escape, the pad's B): nothing else is open while the HUD is drawn.
    if (og.ui.button("Pause", say(world, "hud.pause"), { size: "small", variant: "secondary" }).clicked || og.ui.back()) paused = true;
    sectors(world, r);
    og.ui.minimap("Map", { texture: r.minimap as unknown as Texture, markers, size: og.ui.frameViewport().breakpoint.startsWith("phone") ? 112 : 150 });
  });
  og.ui.speedometer("Speed", {
    speed: Math.round(r.kmh),
    maxSpeed: 180,
    rpm: Math.min(1, r.kmh / 180),
    gear: r.gear === 0 ? "R" : `${r.gear}`,
    unit: say(world, "hud.kmh"),
    anchor: touch ? "bottom" : "bottomRight",
  });
  // Touch: the arrows steer, Brake brakes; the throttle is held for the player.
  let steer = 0;
  let brake = false;
  if (touch) {
    og.ui.anchor("Steer", { anchor: "bottomLeft", gap: S.md }, () => {
      const left = og.ui.actionButton("left", say(world, "hud.left"), { size: 72 });
      const right = og.ui.actionButton("right", say(world, "hud.right"), { size: 72 });
      steer = (right.pressed ? 1 : 0) - (left.pressed ? 1 : 0);
    });
    og.ui.anchor("Pedals", { anchor: "bottomRight", gap: S.md, cross: "center" }, () => {
      brake = og.ui.actionButton("brake", say(world, "hud.brake"), { size: 64 }).pressed;
    });
  }
  r.touchSteer = touch ? steer : 0;
  r.touchThrottle = touch ? (brake ? -1 : 1) : 0;
  if (r.phase === Phase.Countdown) {
    const n = Math.ceil(r.countdown / 60);
    og.ui.anchor("Count", { anchor: "center", direction: "column", cross: "center" }, () => {
      og.ui.label("Number", `${n}`, { font: "display", step: "displayLg", nowrap: true, color: { token: "highlight" } });
      og.ui.label("Hint", say(world, "hud.ready"), { nowrap: true, color: { token: "inkMuted" } });
    });
  }
  const go = r.phase === Phase.Racing && r.clock < 50;
  og.ui.anchor("Go", { anchor: "center", direction: "column", cross: "center" }, () => {
    og.ui.transition("GoText", go, { kind: "pop" }, () => og.ui.label("GoLabel", say(world, "hud.go"), { font: "display", step: "displayLg", nowrap: true, color: { token: "success" } }));
  });
  og.ui.anchor("Wrong", { anchor: "center", offsetY: -120, direction: "column", cross: "center" }, () => {
    og.ui.transition("WrongWay", r.wrongWay > 45, { kind: "pop" }, () => capsule("WrongCapsule", () => og.ui.label("WrongText", say(world, "hud.wrongWay"), { step: "xl", nowrap: true, color: { token: "danger" } })));
  });
  og.ui.anchor("Keys", { anchor: "bottomLeft", direction: "column", gap: S.xs }, () => {
    if (!touch && r.clock < 600 && r.lap === 1) og.ui.label("KeysText", say(world, "hud.keys"), { step: "meta", nowrap: true, color: { token: "inkMuted" } });
    if (!p.ghost) og.ui.label("GhostOff", say(world, "hud.ghostOff"), { step: "meta", nowrap: true, color: { token: "inkFaint" } });
  });
  og.ui.anchor("ReplayTag", { anchor: "left", direction: "column", gap: S.xs }, () => {
    if (r.replaying) capsule("ReplayCapsule", () => og.ui.label("ReplayText", say(world, "hud.replay"), { step: "lg", nowrap: true, color: { token: "highlight" } }));
  });
  return { pause: paused };
}

function board(world: World, r: RaceV): void {
  let rows: [number, string, number, number][] = [];
  try {
    rows = r.board === "" ? [] : (JSON.parse(r.board) as [number, string, number, number][]);
  } catch {
    rows = [];
  }
  const shown = rows.map(([rank, name, secs, me]) => ({ rank, name: me ? say(world, "board.you") : name === "" ? say(world, "board.driver", { n: rank }) : name, time: lapText(Math.round(secs * 60)), you: me === 1 }));
  const vm = { scope: r.boardScope, state: r.boardState === 1 || r.boardState === 3 ? r.boardState : 0, rows: shown };
  for (const e of screen(world, "leaderboard", true, vm)) {
    if (e === "scope") {
      const at = (og.ui.screenVm(world, "leaderboard") as { scope?: number } | null)?.scope ?? r.boardScope;
      r.boardScope = at;
      readBoard(world, r);
    }
    if (e === "retry") readBoard(world, r);
    if (e === "back") r.boardOpen = false;
  }
}

/** The settings tree's pages: what Prefs keeps, every word from the tables. */
function pages(world: World, p: PrefsV): SettingsPage[] {
  const volume = (key: string, label: string, value: number): SettingEntry => ({ key, kind: "slider", label: say(world, label), value, min: 0, max: 1, step: 0.05, unit: "%" });
  const toggle = (key: string, label: string, on: boolean): SettingEntry => ({ key, kind: "toggle", label: say(world, label), on });
  return [
    { key: "audio", label: say(world, "settings.audio"), entries: [volume("master", "settings.master", p.master), volume("music", "settings.music", p.music), volume("effects", "settings.effects", p.effects), toggle("muted", "settings.mute", p.muted)] },
    { key: "graphics", label: say(world, "settings.graphics"), entries: [toggle("reducedMotion", "settings.motion", p.reducedMotion), toggle("highContrast", "settings.contrast", p.highContrast)] },
    { key: "language", label: say(world, "settings.language"), entries: [{ key: "language", kind: "choice", label: say(world, "settings.language"), options: LANGUAGES.map((l) => say(world, l.key)), index: p.language }] },
  ];
}

/** The defaults of a page's entries (Defaults). */
const DEFAULTS: Record<string, Settings> = {
  audio: { masterVolume: 1, musicVolume: 0.8, sfxVolume: 1, muted: false },
  graphics: { reducedMotion: false, highContrast: false },
  language: { language: "en" },
};

function settings(world: World, r: RaceV, p: PrefsV): void {
  // assets/ui/settings.ogt, its Tree slot og.ui.settingsTree; every change is applied at once.
  const asked = screen(world, "settings", r.settingsOpen);
  if (!r.settingsOpen) return;
  if (og.ui.openSlot("settings/Tree")) {
    const res = og.ui.settingsTree(world, "Tree", {
      title: "",
      pages: pages(world, p),
      page: r.settingsPage,
      applyLabel: say(world, "settings.apply"),
      revertLabel: say(world, "settings.revert"),
      defaultsLabel: say(world, "settings.defaults"),
    }, () => {});
    og.ui.close();
    if (res.page !== null) r.settingsPage = res.page;
    const c = res.change;
    if (c !== null) {
      const s: Settings = {};
      if (c.key === "master") s.masterVolume = c.value;
      else if (c.key === "music") s.musicVolume = c.value;
      else if (c.key === "effects") s.sfxVolume = c.value;
      else if (c.key === "muted") s.muted = c.on;
      else if (c.key === "reducedMotion") s.reducedMotion = c.on;
      else if (c.key === "highContrast") s.highContrast = c.on;
      else if (c.key === "language") s.language = LANGUAGES[c.index]?.code;
      takeSettings(world, p, s as Record<string, unknown>);
    }
    if (res.defaults) takeSettings(world, p, (DEFAULTS[r.settingsPage || "audio"] ?? {}) as Record<string, unknown>);
  }
  if (asked.includes("done")) {
    r.settingsOpen = false;
    r.settingsClosedAt = world.tick;
    saveSettings(world, p);
  }
}

/** Stars for the run's best lap against the medal times (seconds). */
export const MEDALS = [40, 37, 35] as const;
export function stars(r: RaceV): number {
  if (r.runBest === 0) return 0;
  const s = r.runBest / 60;
  return MEDALS.filter((m) => s <= m).length;
}

/** The results' count-up: 0.9 s from their opening, ease-out cubic. */
const COUNT_TICKS = 54;

function results(world: World, r: RaceV): "again" | "board" | "title" | "replay" | null {
  const lapsText = [r.lap1, r.lap2, r.lap3].map((t, i) => ({ label: say(world, "results.lap", { n: i + 1 }), value: t === 0 ? "—" : lapText(t) }));
  const open = r.finishedFor - RESULTS_DELAY;
  const k = Math.min(1, Math.max(0, open / COUNT_TICKS));
  // The race's time is the headline, counted up as m:ss.mmm; the best is the session's fastest race before it (none on the first).
  const vm = {
    title: say(world, r.replaying ? (r.replayMatched === 1 ? "results.replayed" : "results.replayOff") : r.newRecord ? "results.record" : "results.finished"),
    score: lapText(k >= 1 ? r.total : Math.round(r.total * (1 - (1 - k) ** 3))),
    best: r.totalBefore > 0 ? say(world, "results.best", { best: lapText(r.totalBefore) }) : "",
    newBest: r.totalBefore > 0 && r.total < r.totalBefore,
    landed: k >= 1,
    stats: [
      { label: say(world, "results.lapRecord"), value: r.record === 0 ? "—" : lapText(r.record) },
      { label: say(world, "results.bestLap"), value: r.runBest === 0 ? "—" : lapText(r.runBest) },
      ...lapsText,
      { label: say(world, "results.rank"), value: r.rank > 0 ? `#${r.rank}` : "—" },
    ],
    replay: r.replayReady,
  };
  const asked = screen(world, "results", true, vm);
  if (og.ui.openSlot("results/Stars")) {
    og.ui.starRating("Rating", { earned: stars(r), max: 3, size: 44, revealAt: og.ui.tick() - Math.max(0, open), name: say(world, "results.stars", { earned: stars(r), max: 3 }) });
    og.ui.close();
  }
  const c = asked.find((e) => e === "again" || e === "board" || e === "title" || e === "replay");
  return (c as "again" | "board" | "title" | "replay" | undefined) ?? null;
}

function toTitle(world: World, r: RaceV): void {
  r.phase = Phase.Title;
  r.paused = false;
  r.boardOpen = false;
  startRaceReset(world, r);
}

function startRaceReset(world: World, r: RaceV): void {
  // The car back on the grid behind the title, the ghost away.
  startRace(world, r);
  r.phase = Phase.Title;
  r.sfx &= ~Sound.Beep;
}

/** The screens, the hold, the platform. */
export const Ui = defineSystem(
  "Ui",
  { reads: [og.Input, og.Gamepad], writes: [Race, Prefs, TapeChunk, Racer, og.Transform, og.Velocity, og.Vehicle, og.Audio] },
  (_q, time, world) => {
    const r = must(world.resource(Race));
    const p = must(world.resource(Prefs));
    if (time.tick === 0) {
      readAll(world, r, p);
      applyPrefs(world, p);
    }
    answers(world, r, p, (s) => takeSettings(world, p, s));
    replayAnswers(world, r);
    // The replay's start: saved by the first system of the tick, so the restored world runs this whole tick again.
    if (r.phase === Phase.Countdown && r.countdown === SNAPSHOT_AT + 1 && !r.replaying && !r.held) saveStart(world, r);
    const racing = r.phase === Phase.Countdown || r.phase === Phase.Racing;
    const hold = racing && (r.paused || r.settingsOpen || r.replayWait);
    if (hold !== r.held) {
      must(og.time.setScale(world, hold ? 0 : 1));
      r.held = hold;
    }
    if (r.phase === Phase.Finished) r.finishedFor += 1;
    if (r.wrongWay === 46) og.ui.toast(world, say(world, "hud.wrongWay"), { tone: "danger" });

    const want = r.phase === Phase.Title ? (r.boardOpen ? "board" : "title") : r.phase === Phase.Finished && r.finishedFor >= RESULTS_DELAY ? (r.boardOpen ? "board" : "results") : "race";
    const shown = og.ui.screenTransition("Screens", want, { kind: p.reducedMotion ? "fade" : "wipe", durationMs: 600 }).shown;
    if (shown === "title") {
      og.ui.setNavigation(true);
      const sub = r.record > 0 ? say(world, "title.record", { time: lapText(r.record) }) : say(world, "title.tagline");
      // assets/ui/main-menu.ogt: Time trial, Leaderboard, Settings.
      const asked = screen(world, "main-menu", true, { sub });
      if (!r.settingsOpen && r.phase === Phase.Title) {
        if (asked.includes("start")) startRace(world, r);
        if (asked.includes("board")) {
          r.boardOpen = true;
          readBoard(world, r);
        }
        if (asked.includes("settings")) r.settingsOpen = true;
      }
    } else if (shown === "board") {
      og.ui.setNavigation(true);
      board(world, r);
    } else if (shown === "results") {
      og.ui.setNavigation(true);
      const c = results(world, r);
      if (c === "again") startRace(world, r);
      if (c === "title") toTitle(world, r);
      if (c === "replay") watch(world, r);
      if (c === "board") {
        r.boardOpen = true;
        readBoard(world, r);
      }
    } else {
      // Not drawn under pause or the settings: the race's frames show those over the bare road.
      const h = r.paused || r.settingsOpen ? { pause: false } : hud(world, r, p);
      if (racing && !r.paused && !r.settingsOpen && (h.pause || must(world.resource(og.Input)).pressed("KeyP"))) r.paused = true;
      // assets/ui/pause.ogt: Resume (and Back), Restart, the ghost, Settings, Quit to title.
      for (const e of screen(world, "pause", r.paused && !r.settingsOpen, { ghost: say(world, p.ghost ? "pause.ghostOn" : "pause.ghostOff") })) {
        if (e === "resume") r.paused = false;
        if (e === "restart") startRace(world, r);
        if (e === "ghost") {
          p.ghost = !p.ghost;
          if (!p.ghost) r.ghostActive = false;
          saveSettings(world, p);
        }
        if (e === "settings") r.settingsOpen = true;
        if (e === "quit") toTitle(world, r);
      }
    }
    // The screen documents of the screens not shown leave the layer stack.
    if (shown !== "title") screen(world, "main-menu", false);
    if (shown !== "board") screen(world, "leaderboard", false);
    if (shown !== "results") screen(world, "results", false);
    if (shown !== "race") screen(world, "pause", false);
    settings(world, r, p);
    // The accessibility counts, for the locales playtest: every half second, and each tick a dialog is open.
    if (!r.settingsOpen && !r.paused && time.tick % 30 !== 0) return;
    const tree = og.ui.a11y();
    const unnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "").length;
    if (unnamed !== r.a11yUnnamed) r.a11yUnnamed = unnamed;
    if (tree.nodes.length !== r.a11yNodes) r.a11yNodes = tree.nodes.length;
  },
);
