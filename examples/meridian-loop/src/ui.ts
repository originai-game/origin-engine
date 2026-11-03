// Meridian Loop's screens, on og.ui with the neon style (assets/styles/meridian,
// brought by the look), laid out as the game shipped them:
//
// - the title, pause, the settings, the results and the leaderboard: the
//   game's own screen documents (assets/ui/*.ogt, ejected from the library's
//   main-menu, pause, settings, results and leaderboard with `og ui eject`),
//   their slots filled here: Time trial, Leaderboard (the fastest laps and
//   the ones round the player, its loading, error and empty states honest),
//   Settings (its tabs and rows), every word from the string tables, English
//   and German;
// - the race: the racing HUD as a time trial (src/hud.ts: no place, no
//   boost meter: the lap, the lap's time, the record, the sectors, the
//   minimap with the car and the ghost, the speed and gear), touch steering
//   on a phone; the countdown, Wrong way;
// - pause (Resume, Restart, the ghost on or off, Settings, Quit), the
//   results (the race's time counting up, the session's best before it, the
//   stars), toasts; a wipe between the screens (a fade with reduced motion).
//
// The Ui system also holds time under a menu, and takes og.platform's
// answers (src/save.ts).

import { og, defineSystem } from "@origingame/engine";
import type { MapMarker, Texture, Value, World } from "@origingame/engine";
import { racingHud, type HudSplit } from "./hud.ts";
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

/** The lap's sectors as the HUD's splits: each one's time once crossed, and its delta against the record's sector. */
function splits(world: World, r: RaceV): HudSplit[] {
  const mine = [r.split1, r.split2 > 0 ? r.split2 - r.split1 : 0, 0];
  const recs = [r.recordS1, r.recordS2, r.recordS3];
  return mine.map((t, i) => ({
    label: say(world, "hud.split", { n: i + 1 }),
    time: t === 0 ? null : seconds(t),
    delta: t === 0 || recs[i]! === 0 ? null : seconds(t - recs[i]!),
  }));
}

function hud(world: World, r: RaceV, p: PrefsV): { pause: boolean } {
  const touch = og.ui.frameViewport().coarsePointer;
  const car = must(world.get(entity(world, "car"), og.Transform)).position;
  const markers: MapMarker[] = [{ ...mapPoint(car.x, car.z), player: true }];
  if (r.ghostActive) {
    const gh = must(world.get(entity(world, "ghost"), og.Transform)).position;
    markers.push({ ...mapPoint(gh.x, gh.z), tone: "info" });
  }
  const lapTicks = r.phase === Phase.Racing ? r.clock - r.lapStart : 0;
  // src/hud.ts: the racing HUD as the game shipped it, a time trial's (no place, no boost).
  const h = racingHud("Hud", {
    speed: Math.round(r.kmh),
    maxSpeed: 180,
    gear: r.gear === 0 ? "R" : `${r.gear}`,
    lap: Math.min(LAPS, r.lap),
    laps: LAPS,
    time: lapTicks / 60,
    // The last split's (or the line's) delta against the record, for three seconds.
    delta: r.deltaShow > 0 && r.deltaAt > 0 ? seconds(r.delta) : null,
    best: r.record > 0 ? seconds(r.record) : null,
    splits: splits(world, r),
    map: { texture: r.minimap as unknown as Texture, markers, size: 150 },
    touch,
    // Every word of the HUD from the tables.
    unit: say(world, "hud.kmh"),
    lapLabel: say(world, "hud.lap"),
    bestLabel: say(world, "hud.best"),
    pauseLabel: say(world, "hud.pause"),
    brakeLabel: say(world, "hud.brake"),
    leftLabel: say(world, "hud.left"),
    rightLabel: say(world, "hud.right"),
  });
  // Touch: the arrows steer, Brake brakes; the throttle is held for the player.
  r.touchSteer = touch ? h.moveX : 0;
  r.touchThrottle = touch ? (h.held.includes("brake") ? -1 : 1) : 0;
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
  return { pause: h.paused };
}

/** The medal inks of the board's first three ranks. */
const MEDAL_INKS = ["warn", "inkMuted", "highlight"] as const;

function board(world: World, r: RaceV): void {
  let rows: [number, string, number, number][] = [];
  try {
    rows = r.board === "" ? [] : (JSON.parse(r.board) as [number, string, number, number][]);
  } catch {
    rows = [];
  }
  const entries = rows.map(([rank, name, secs, me]) => ({ rank, name: me ? say(world, "board.you") : name === "" ? say(world, "board.driver", { n: rank }) : name, time: lapText(Math.round(secs * 60)), you: me === 1 }));
  // assets/ui/leaderboard.ogt, its Body slot the scopes and the laps as the game shipped them.
  const asked = screen(world, "leaderboard", true);
  if (og.ui.openSlot("leaderboard/Body")) {
    const theme = og.ui.frameTheme();
    const vp = og.ui.frameViewport();
    const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
    const tabs = og.ui.tabs("Scopes", [say(world, "board.fastest"), say(world, "board.around")], r.boardScope, { fill: true, width: { kind: "grow" } });
    const height = px(phone ? 380 : 360);
    if (r.boardState === 1) {
      // Honest loading: skeleton rows, never invented laps.
      og.ui.column("Loading", { width: { kind: "grow" }, gap: S.md, height }, () => {
        for (let i = 0; i < 6; i++) og.ui.skeleton(og.ui.idi("Row", i), { height: theme.sizes.controlSm });
      });
    } else if (r.boardState === 3) {
      og.ui.column("Error", { width: { kind: "grow" }, gap: S.md, height, align: "center", cross: "center" }, () => {
        og.ui.label("Message", say(world, "board.error"), { color: { token: "danger" }, align: "center", width: { kind: "grow" } });
        if (og.ui.button("Retry", say(world, "board.retry"), {}).clicked) readBoard(world, r);
      });
    } else if (entries.length === 0) {
      og.ui.column("Empty", { width: { kind: "grow" }, height, align: "center", cross: "center" }, () => {
        og.ui.label("Message", say(world, "board.empty"), { color: { token: "inkMuted" }, align: "center", width: { kind: "grow" } });
      });
    } else {
      const you = entries.findIndex((e) => e.you);
      og.ui.list("Rows", { count: entries.length, rowHeight: theme.sizes.row, height, width: { kind: "grow" }, selected: you >= 0 ? you : null }, (i) => {
        const e = entries[i]!;
        og.ui.label("Rank", `${e.rank}`, { width: px(40), font: "mono", step: e.rank <= 3 ? "lg" : "body", color: { token: e.rank <= 3 ? MEDAL_INKS[e.rank - 1]! : "inkMuted" }, nowrap: true });
        og.ui.label("Name", e.name, { width: { kind: "grow" }, nowrap: true });
        og.ui.label("Score", e.time, { font: "mono", nowrap: true });
      });
    }
    og.ui.close();
    if (tabs.response.changed) {
      r.boardScope = tabs.selected;
      readBoard(world, r);
    }
  }
  if (asked.includes("back")) r.boardOpen = false;
}

/** The settings' pages, in their tabs' order. */
const PAGES = ["audio", "graphics", "controls", "language"] as const;

/** A labelled slider row, its value at the end, as the settings shipped it (the label 140 px wide, 96 on a phone). */
function sliderRow(id: string, label: string, value: number, desc: { min?: number; max?: number; step?: number }, shown: string, phone: boolean): number {
  let out = value;
  og.ui.row(id, { width: { kind: "grow" }, gap: S.md, cross: "center" }, () => {
    og.ui.label("Label", label, { width: px(phone ? 96 : 140), nowrap: true });
    out = og.ui.slider("Slider", value, desc).value;
    og.ui.label("Value", shown, { width: px(48), align: "end", font: "mono", color: { token: "inkMuted" }, nowrap: true });
  });
  return out;
}

function settings(world: World, r: RaceV, p: PrefsV): void {
  // assets/ui/settings.ogt, its Page slot the tabs and the page's rows; every change is applied at once.
  const asked = screen(world, "settings", r.settingsOpen);
  if (!r.settingsOpen) return;
  if (og.ui.openSlot("settings/Page")) {
    const vp = og.ui.frameViewport();
    const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
    const labels = PAGES.map((k) => say(world, `settings.${k}`));
    let tab = Math.max(0, PAGES.indexOf(r.settingsPage as (typeof PAGES)[number]));
    // Tabs on a wide surface, a select on a phone.
    tab = (phone ? og.ui.select("Section", labels, tab, { width: { kind: "grow" }, label: "Section" }) : og.ui.tabs("Section", labels, tab, { fill: true, width: { kind: "grow" }, label: "Section" })).selected;
    r.settingsPage = PAGES[tab]!;
    const s: Settings = {};
    const pct = (v: number): string => `${Math.round(v * 100)}%`;
    // The body keeps at least 260 px (300 on a phone) where the screen has room and scrolls past it; its right
    // padding keeps the values clear of the scroll bar.
    const floor = vp.height >= 600 ? (phone ? 300 : 260) : 0;
    og.ui.scroll("Body", { width: { kind: "grow" }, height: { kind: "fit", min: floor }, gap: S.sm, padding: { left: 0, top: 0, right: 8, bottom: 0 } }, () => {
      switch (PAGES[tab]) {
        case "audio": {
          const volume = (id: string, key: string, value: number, set: (v: number) => void): void => {
            const v = sliderRow(id, say(world, key), value, { step: 0.05 }, pct(value), phone);
            if (v !== value) set(v);
          };
          volume("Master", "settings.master", p.master, (v) => (s.masterVolume = v));
          volume("Music", "settings.music", p.music, (v) => (s.musicVolume = v));
          volume("Effects", "settings.effects", p.effects, (v) => (s.sfxVolume = v));
          const muted = og.ui.toggle("Mute", p.muted, say(world, "settings.mute")).on;
          if (muted !== p.muted) s.muted = muted;
          break;
        }
        case "graphics": {
          // The quality and the frame rate were the settings' own (the game keeps neither): shown at their defaults.
          og.ui.label("QualityLabel", say(world, "settings.quality"), { step: "meta", color: { token: "inkMuted" } });
          og.ui.radio("Quality", [say(world, "settings.low"), say(world, "settings.medium"), say(world, "settings.high")], 1, { horizontal: !phone, label: say(world, "settings.quality") });
          og.ui.toggle("Fps", false, say(world, "settings.fps"));
          const motion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
          if (motion !== p.reducedMotion) s.reducedMotion = motion;
          const contrast = og.ui.toggle("Contrast", p.highContrast, say(world, "settings.contrast")).on;
          if (contrast !== p.highContrast) s.highContrast = contrast;
          break;
        }
        case "controls": {
          // Likewise the look's sensitivity and inversion and the pad's vibration: the game keeps none of them.
          sliderRow("Sensitivity", say(world, "settings.sensitivity"), 1, { min: 0.1, max: 3, step: 0.1 }, "1.0", phone);
          og.ui.checkbox("Invert", false, say(world, "settings.invert"));
          og.ui.checkbox("Vibration", true, say(world, "settings.vibration"));
          break;
        }
        case "language": {
          const at = og.ui.radio("Languages", LANGUAGES.map((l) => say(world, l.key)), p.language, { label: "Languages" }).selected;
          if (at !== p.language) s.language = LANGUAGES[at]?.code;
          break;
        }
      }
    });
    og.ui.close();
    if (Object.keys(s).length > 0) takeSettings(world, p, s as Record<string, unknown>);
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

/** The results' stars: each one scales in 0.3 s after the last from the opening, earned ones in the primary ink. */
function starRow(earned: number, max: number, since: number, size: number): void {
  og.ui.row("Stars", { gap: S.sm, cross: "center" }, () => {
    for (let k = 0; k < max; k++) {
      const shown = og.ui.seconds() - since >= 0.3 * (k + 1);
      const on = k < earned;
      og.ui.transition(og.ui.idi("Star", k), shown, { kind: "scale", instant: false }, () => {
        og.ui.icon("Mark", "star", { size, tint: { token: on ? "primary" : "controlStrong" } });
        if (on) og.ui.burst({ delayMs: 120 });
      });
      if (!shown) og.ui.spacer(og.ui.idi("Gap", k), px(size));
    }
  });
}

/** A finished time as m:ss.mmm, cut to the millisecond. */
function lapTime(secs: number): string {
  const ms = Math.floor(Math.max(0, secs) * 1000 + 1e-6);
  return `${Math.floor(ms / 60000)}:${(Math.floor(ms / 1000) % 60).toString().padStart(2, "0")}.${(ms % 1000).toString().padStart(3, "0")}`;
}

/** The running count-up as m:ss.mmm, its milliseconds floored. */
function clockText(secs: number): string {
  const s = Math.max(0, secs);
  const m = Math.floor(s / 60);
  const rest = s - m * 60;
  const whole = Math.floor(rest);
  return `${m}:${whole.toString().padStart(2, "0")}.${Math.floor((rest - whole) * 1000).toString().padStart(3, "0")}`;
}

function results(world: World, r: RaceV): "again" | "board" | "title" | "replay" | null {
  const lapsText = [r.lap1, r.lap2, r.lap3].map((t, i) => ({ label: say(world, "results.lap", { n: i + 1 }), value: t === 0 ? "—" : lapText(t) }));
  const stats = [
    { label: say(world, "results.lapRecord"), value: r.record === 0 ? "—" : lapText(r.record) },
    { label: say(world, "results.bestLap"), value: r.runBest === 0 ? "—" : lapText(r.runBest) },
    ...lapsText,
    { label: say(world, "results.rank"), value: r.rank > 0 ? `#${r.rank}` : "—" },
  ];
  const title = say(world, r.replaying ? (r.replayMatched === 1 ? "results.replayed" : "results.replayOff") : r.newRecord ? "results.record" : "results.finished");
  const asked = screen(world, "results", true, { title, replay: r.replayReady });
  // When the results opened, in UI seconds: the count-up and the stars run from there.
  let since = og.ui.memory("Results.since", -1);
  if (since < 0) since = og.ui.seconds();
  og.ui.remember("Results.since", since);
  if (og.ui.openSlot("results/Body")) {
    const elapsed = og.ui.seconds() - since;
    // The race's time is the headline, counting up as m:ss.mmm over 0.9 s (ease-out cubic); the best is the
    // session's fastest race before it (none on the first), and Best race pops in when the count lands.
    const score = seconds(r.total);
    const prior = seconds(r.totalBefore);
    const newBest = r.totalBefore > 0 && r.total < r.totalBefore;
    const best = newBest ? Math.min(prior, score) : prior;
    og.ui.row("StarRow", { width: { kind: "grow" }, align: "center" }, () => starRow(stars(r), 3, since, 44));
    og.ui.column("ScoreBlock", { width: { kind: "grow" }, cross: "center", gap: 2 }, () => {
      og.ui.label("ScoreLabel", say(world, "results.total"), { step: "meta", color: { token: "inkMuted" }, align: "center", width: { kind: "grow" }, nowrap: true });
      const k = Math.min(1, Math.max(0, elapsed / 0.9));
      og.ui.label("Score", k >= 1 ? lapTime(score) : clockText(score * (1 - (1 - k) ** 3)), { step: "displayLg", font: "display", align: "center", width: { kind: "grow" }, nowrap: true });
      if (best > 0 || newBest) {
        og.ui.row("BestRow", { gap: S.sm, cross: "center" }, () => {
          if (best > 0) og.ui.label("Best", say(world, "results.best", { best: lapTime(best) }), { step: "meta", color: { token: "inkMuted" }, nowrap: true });
          if (newBest) og.ui.transition("Reveal", elapsed >= 0.9, { kind: "pop" }, () => og.ui.badge("NewBest", say(world, "results.newBest"), "primary"));
        });
      }
    });
    og.ui.column("Stats", { width: { kind: "grow" }, gap: 0 }, () => {
      stats.forEach((st, i) => og.ui.keyValue(og.ui.idi("Stat", i), st.label, st.value, true));
    });
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
    if (shown !== "results") {
      screen(world, "results", false);
      og.ui.remember("Results.since", -1);
    }
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
