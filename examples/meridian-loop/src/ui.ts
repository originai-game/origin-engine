// Meridian Loop's screens, on og.ui with the neon skin (assets/skins/meridian,
// brought by the look):
//
// - the title (og.ui.mainMenu): Time trial, Leaderboard
//   (og.ui.leaderboardScreen: the fastest laps and the ones round the
//   player, its loading, error and empty states honest), Settings
//   (og.ui.settingsScreen, every word from the string tables, English and
//   German); the record under the title;
// - the race: og.ui.racingHud (speed, gear, lap, the lap time, the record,
//   the mini-map with the car and the ghost; touch steering on a phone),
//   the three sector times and the last split's delta against the record
//   top centre, the countdown, Wrong way;
// - pause (og.ui.pauseMenu: Resume, Restart, the ghost on or off, Settings,
//   Quit), the results (og.ui.resultsScreen), toasts; a wipe between the
//   screens (a fade with reduced motion).
//
// The Ui system also holds time under a menu, and takes og.platform's
// answers (src/save.ts).

import { og, defineSystem } from "@origingame/engine";
import type { GameSettings, Value, World } from "@origingame/engine";
import { Phase, Prefs, Race, Sound, TapeChunk } from "./components.ts";
import { deltaText, lapText, startRace } from "./race.ts";
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

function settingsOf(p: PrefsV): GameSettings {
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
  must(og.platform.saveSettings(world, { ...settingsOf(p), ghost: p.ghost } as GameSettings));
  p.saves += 1;
}

/** Seconds as s.mmm, for a split. */
function secs(ticks: number): string {
  const ms = Math.round((ticks * 1000) / 60);
  return `${Math.floor(ms / 1000)}.${String(ms % 1000).padStart(3, "0")}`;
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.sm, cross: "center", padding: { left: S.md, top: S.xs, right: S.md, bottom: S.xs }, frame: "capsule", background: { token: "surface" } }, build);
}

/** The sector times: green under the record's, red over, accent while it is the one being driven. */
function sectors(world: World, r: RaceV): void {
  const mine = [r.split1, r.split2 > 0 ? r.split2 - r.split1 : 0, 0];
  const recs = [r.recordS1, r.recordS2, r.recordS3];
  const driving = r.split1 === 0 ? 0 : r.split2 === 0 ? 1 : 2;
  og.ui.anchor("Sectors", { anchor: "top", offsetY: 12, direction: "column", cross: "center", gap: S.xs }, () => {
    capsule("Splits", () => {
      for (let i = 0; i < 3; i++) {
        const t = mine[i]!;
        const tone = t === 0 ? (i === driving && r.phase === Phase.Racing ? "accent" : "fgSubtle") : recs[i]! === 0 ? "fg" : t <= recs[i]! ? "success" : "danger";
        og.ui.label(`S${i + 1}`, say(world, "hud.sector", { n: i + 1, time: t === 0 ? "--.---" : secs(t) }), { font: "mono", nowrap: true, color: { token: tone } });
      }
    });
    og.ui.transition("Delta", r.deltaShow > 0 && r.deltaAt > 0, { kind: "pop" }, () => {
      capsule("DeltaCapsule", () => {
        og.ui.label("DeltaText", say(world, r.deltaAt === 3 ? "hud.deltaLap" : "hud.deltaSplit", { n: r.deltaAt, delta: deltaText(r.delta) }), { font: "mono", step: "xl", nowrap: true, color: { token: r.delta <= 0 ? "success" : "danger" } });
      });
    });
  });
}

function hud(world: World, r: RaceV, p: PrefsV): { pause: boolean } {
  og.ui.setNavigation(false);
  const touch = og.ui.frameViewport().coarsePointer;
  const car = must(world.get(entity(world, "car"), og.Transform)).position;
  const markers = [{ ...mapPoint(car.x, car.z), player: true }];
  if (r.ghostActive) {
    const gh = must(world.get(entity(world, "ghost"), og.Transform)).position;
    markers.push({ ...mapPoint(gh.x, gh.z), player: false, tone: "info" } as typeof markers[number] & { tone: "info" });
  }
  const lapTicks = r.phase === Phase.Racing ? r.clock - r.lapStart : 0;
  const h = og.ui.racingHud("Hud", {
    speed: Math.round(r.kmh),
    maxSpeed: 180,
    unit: say(world, "hud.kmh"),
    gear: r.gear === 0 ? "R" : `${r.gear}`,
    lap: Math.min(LAPS, r.lap),
    laps: LAPS,
    position: 1,
    racers: 1,
    time: lapTicks / 60,
    best: r.record > 0 ? r.record / 60 : null,
    map: { texture: r.minimap as never, markers, size: 150 },
    touch,
  });
  // Touch: the arrows steer, Brake brakes; the throttle is held for the player.
  r.touchSteer = touch ? h.moveX : 0;
  r.touchThrottle = touch ? (h.held.includes("brake") ? -1 : 1) : 0;
  sectors(world, r);
  if (r.phase === Phase.Countdown) {
    const n = Math.ceil(r.countdown / 60);
    og.ui.anchor("Count", { anchor: "center", direction: "column", cross: "center" }, () => {
      og.ui.label("Number", `${n}`, { font: "display", step: "displayLg", nowrap: true, color: { token: "accentInk" } });
      og.ui.label("Hint", say(world, "hud.ready"), { nowrap: true, color: { token: "fgMuted" } });
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
    if (!touch && r.clock < 600 && r.lap === 1) og.ui.label("KeysText", say(world, "hud.keys"), { step: "meta", nowrap: true, color: { token: "fgMuted" } });
    if (!p.ghost) og.ui.label("GhostOff", say(world, "hud.ghostOff"), { step: "meta", nowrap: true, color: { token: "fgSubtle" } });
  });
  return { pause: h.paused };
}

function board(world: World, r: RaceV): void {
  let rows: [number, string, number, number][] = [];
  try {
    rows = r.board === "" ? [] : (JSON.parse(r.board) as [number, string, number, number][]);
  } catch {
    rows = [];
  }
  const entries = rows.map(([rank, name, seconds, me]) => ({ rank, name: me ? say(world, "board.you") : name === "" ? say(world, "board.driver", { n: rank }) : name, score: lapText(Math.round(seconds * 60)), player: me === 1 }));
  const res = og.ui.leaderboardScreen("Board", {
    title: say(world, "board.title"),
    scopes: [say(world, "board.fastest"), say(world, "board.around")],
    entries,
    loading: r.boardState === 1,
    error: r.boardState === 3 ? say(world, "board.error") : "",
    empty: say(world, "board.empty"),
    retryLabel: say(world, "board.retry"),
  });
  if (res.changed) {
    r.boardScope = res.scope;
    readBoard(world, r);
  }
  if (res.retry) readBoard(world, r);
  if (res.closed) r.boardOpen = false;
}

function settings(world: World, r: RaceV, p: PrefsV): void {
  if (!r.settingsOpen && world.tick - r.settingsClosedAt > 30) {
    og.ui.settingsScreen("Settings", { open: false });
    return;
  }
  const res = og.ui.settingsScreen("Settings", {
    open: r.settingsOpen,
    settings: settingsOf(p),
    languages: LANGUAGES.map((l) => ({ code: l.code, label: say(world, l.key) })),
    title: say(world, "settings.title"),
    audioLabel: say(world, "settings.audio"),
    graphicsLabel: say(world, "settings.graphics"),
    controlsLabel: say(world, "settings.controls"),
    languageLabel: say(world, "settings.language"),
    masterLabel: say(world, "settings.master"),
    musicLabel: say(world, "settings.music"),
    effectsLabel: say(world, "settings.effects"),
    muteLabel: say(world, "settings.mute"),
    qualityLabel: say(world, "settings.quality"),
    lowLabel: say(world, "settings.low"),
    mediumLabel: say(world, "settings.medium"),
    highLabel: say(world, "settings.high"),
    fpsLabel: say(world, "settings.fps"),
    motionLabel: say(world, "settings.motion"),
    contrastLabel: say(world, "settings.contrast"),
    sensitivityLabel: say(world, "settings.sensitivity"),
    invertLabel: say(world, "settings.invert"),
    vibrationLabel: say(world, "settings.vibration"),
    doneLabel: say(world, "settings.done"),
  });
  if (!r.settingsOpen) return;
  if (res.changed) takeSettings(world, p, res.settings as unknown as Record<string, unknown>);
  if (res.closed) {
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

function results(world: World, r: RaceV): "again" | "board" | "title" | null {
  const lapsText = [r.lap1, r.lap2, r.lap3].map((t, i) => ({ label: say(world, "results.lap", { n: i + 1 }), value: t === 0 ? "—" : lapText(t) }));
  const res = og.ui.resultsScreen("Results", {
    open: r.phase === Phase.Finished && r.finishedFor >= RESULTS_DELAY,
    title: say(world, r.newRecord ? "results.record" : "results.finished"),
    score: Math.round((r.total * 1000) / 60) / 1000,
    best: r.record / 60,
    newBest: r.newRecord,
    bestLabel: say(world, "results.best", { best: r.record > 0 ? lapText(r.record) : "—" }),
    newBestLabel: say(world, "results.newBest"),
    stars: stars(r),
    maxStars: 3,
    stats: [
      { label: say(world, "results.total"), value: lapText(r.total) },
      { label: say(world, "results.bestLap"), value: r.runBest === 0 ? "—" : lapText(r.runBest) },
      ...lapsText,
      { label: say(world, "results.rank"), value: r.rank > 0 ? `#${r.rank}` : "—" },
    ],
    actions: [
      { id: "again", label: say(world, "results.again"), primary: true },
      { id: "board", label: say(world, "results.board") },
      { id: "title", label: say(world, "results.title") },
    ],
  });
  return res.selected === "again" || res.selected === "board" || res.selected === "title" ? res.selected : null;
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
  { reads: [og.Input, og.Gamepad], writes: [Race, Prefs, TapeChunk, og.Transform, og.Velocity, og.Vehicle, og.Audio] },
  (_q, time, world) => {
    const r = must(world.resource(Race));
    const p = must(world.resource(Prefs));
    if (time.tick === 0) {
      readAll(world, r, p);
      applyPrefs(world, p);
    }
    answers(world, r, p, (s) => takeSettings(world, p, s));
    const racing = r.phase === Phase.Countdown || r.phase === Phase.Racing;
    const hold = racing && (r.paused || r.settingsOpen);
    if (hold !== r.held) {
      must(og.time.setScale(world, hold ? 0 : 1));
      r.held = hold;
    }
    if (r.phase === Phase.Finished) r.finishedFor += 1;
    if (r.wrongWay === 46) og.ui.toast(world, say(world, "hud.wrongWay"), { tone: "danger" });

    const screen = r.phase === Phase.Title ? (r.boardOpen ? "board" : "title") : r.phase === Phase.Finished && r.finishedFor >= RESULTS_DELAY ? (r.boardOpen ? "board" : "results") : "race";
    const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "wipe", durationMs: 600 }).shown;
    if (shown === "title") {
      og.ui.setNavigation(true);
      const items = [
        { id: "start", label: say(world, "title.start"), primary: true },
        { id: "board", label: say(world, "title.board") },
        { id: "settings", label: say(world, "title.settings") },
      ];
      const sub = r.record > 0 ? say(world, "title.record", { time: lapText(r.record) }) : say(world, "title.tagline");
      const m = og.ui.mainMenu("Title", { title: say(world, "title.name"), subtitle: sub, items, footer: say(world, "title.footer") });
      if (!r.settingsOpen && r.phase === Phase.Title) {
        if (m.selected === "start") startRace(world, r);
        if (m.selected === "board") {
          r.boardOpen = true;
          readBoard(world, r);
        }
        if (m.selected === "settings") r.settingsOpen = true;
      }
    } else if (shown === "board") {
      og.ui.setNavigation(true);
      board(world, r);
    } else if (shown === "results") {
      og.ui.setNavigation(true);
      const c = results(world, r);
      if (c === "again") startRace(world, r);
      if (c === "title") toTitle(world, r);
      if (c === "board") {
        r.boardOpen = true;
        readBoard(world, r);
      }
    } else {
      const h = hud(world, r, p);
      if (racing && !r.paused && !r.settingsOpen && (h.pause || must(world.resource(og.Input)).pressed("KeyP"))) r.paused = true;
      const m = og.ui.pauseMenu("Pause", {
        open: r.paused && !r.settingsOpen,
        title: say(world, "pause.title"),
        items: [
          { id: "resume", label: say(world, "pause.resume"), primary: true },
          { id: "restart", label: say(world, "pause.restart") },
          { id: "ghost", label: say(world, p.ghost ? "pause.ghostOn" : "pause.ghostOff") },
          { id: "settings", label: say(world, "pause.settings") },
          { id: "quit", label: say(world, "pause.quit") },
        ],
      });
      if (m.selected === "resume") r.paused = false;
      if (m.selected === "restart") startRace(world, r);
      if (m.selected === "ghost") {
        p.ghost = !p.ghost;
        if (!p.ghost) r.ghostActive = false;
        saveSettings(world, p);
      }
      if (m.selected === "settings") r.settingsOpen = true;
      if (m.selected === "quit") toTitle(world, r);
    }
    settings(world, r, p);
    // The accessibility counts, for the locales playtest: every half second, and each tick a dialog is open.
    if (!r.settingsOpen && !r.paused && time.tick % 30 !== 0) return;
    const tree = og.ui.a11y();
    const unnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "").length;
    if (unnamed !== r.a11yUnnamed) r.a11yUnnamed = unnamed;
    if (tree.nodes.length !== r.a11yNodes) r.a11yNodes = tree.nodes.length;
  },
);
