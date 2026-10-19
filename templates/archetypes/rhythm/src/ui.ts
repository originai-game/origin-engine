// Three-Lane Beat's screens on og.ui and the skin in assets/skins/lanebeat
// (the engine's sci-fi pack): the title, the HUD (the score and the groove
// top left, Pause top right, the combo and its multiplier beside the
// highway), the pause menu, the settings with the offset, the calibration
// and the results. Screens change through a wipe (a fade with reduced
// motion). Every string is a key of assets/strings/*.ogt.
//
// The Ui system also starts a play (the count-in, then bar 1), pauses it
// (the band stops, the clock freezes) and resumes it on the bar line of
// its two-bar phrase after a new count-in, ends it with the results, and
// keeps the settings and the best score in og.platform.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, Phase, Prefs, Tuning } from "./components.ts";
import { beatAt, beatLength, lazyTuning, must, worldTime } from "./systems.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** The tokens' spacing scale in CSS px (og.ui.frameTheme answers it too, but builds the whole theme each call). */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24 } as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** A new play: the counts cleared, the count-in from the next beat, then bar 1 (or `fromBar`). */
export function begin(world: World, g: GameV, fromBar = 1): void {
  const t = lazyTuning(world);
  const len = beatLength(g);
  const now = worldTime(world);
  g.phase = Phase.Play;
  g.paused = false;
  g.pausedAt = -1;
  g.score = 0;
  g.combo = 0;
  g.maxCombo = 0;
  g.perfect = 0;
  g.great = 0;
  g.good = 0;
  g.miss = 0;
  g.hits = 0;
  g.judged = 0;
  g.taps = 0;
  g.scheduled = 0;
  g.groove = t.grooveStart;
  g.failed = false;
  g.cleared = false;
  g.over = false;
  g.endAt = 0;
  g.newBest = false;
  g.lastJudgement = -1;
  g.countShown = -1;
  g.resetNotes = true;
  g.stopBand = true;
  // The first bar starts countInBeats after the next whole tick: the clicks lead in.
  const from = (Math.max(1, Math.min(g.bars, fromBar)) - 1) * 4;
  g.fromBeat = from;
  g.countTo = Math.ceil((now + 0.25) * 60) / 60 + t.countInBeats * len;
  g.songStart = g.countTo - from * len;
  g.bandFrom = from / 4 + 1;
  g.scheduleUpTo = from + t.scrollBeats + 1;
}

/** Pause: the clock freezes where it is, the band stops. */
export function pause(world: World, g: GameV): void {
  if (g.phase !== Phase.Play || g.paused || g.over) return;
  g.paused = true;
  g.pausedAt = worldTime(world);
  g.stopBand = true;
}

/** Resume on the bar line of the two-bar phrase it was paused in, after a new count-in. */
export function resume(world: World, g: GameV): void {
  if (!g.paused) return;
  const t = lazyTuning(world);
  const len = beatLength(g);
  const at = Math.max(0, beatAt(g, g.pausedAt));
  const from = Math.floor(at / 8) * 8;
  const now = worldTime(world);
  const countTo = Math.ceil((now + 0.25) * 60) / 60 + t.countInBeats * len;
  g.songStart = countTo - from * len;
  g.countTo = countTo;
  g.bandFrom = from / 4 + 1;
  g.scheduleUpTo = from + t.scrollBeats + 1;
  g.countShown = -1;
  g.paused = false;
  g.pausedAt = -1;
}

/** Calibration: the clock clicks every beat from the next one; taps measure the offset. */
export function calibrate(world: World, g: GameV): void {
  g.phase = Phase.Calibrate;
  g.calTaps = 0;
  g.calSum = 0;
  g.stopBand = true;
  g.songStart = Math.ceil((worldTime(world) + 0.5) * 60) / 60;
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
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, language: LANGUAGES[p.language]!.code, reducedMotion: p.reducedMotion, offsetMs: p.offsetMs };
}

function saveSettings(world: World, p: PrefsV): void {
  must(og.platform.saveSettings(world, prefsValue(p)));
  p.saves += 1;
}

/** og.platform: the settings and the best score asked for on tick 0; the best written at the revision read. */
function platform(world: World, g: GameV, p: PrefsV): void {
  if (world.tick === 0) {
    p.loadRequest = og.platform.loadSettings(world);
    const r = og.platform.storeGet(world, "best");
    g.bestRequest = r.ok ? r.value : 0;
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
        if (typeof s.offsetMs === "number") p.offsetMs = s.offsetMs;
        const at = LANGUAGES.findIndex((l) => l.code === s.language);
        if (at >= 0) setLanguage(world, p, at);
        applyPrefs(world, p);
      }
    }
  }
  if (g.bestRequest !== 0) {
    const r = og.platform.result(world, g.bestRequest);
    if (r !== null) {
      g.bestRequest = 0;
      const d = (r.ok ? r.data : null) as { value?: number | null; revision?: number } | null;
      if (typeof d?.value === "number") g.best = Math.max(g.best, d.value);
      if (typeof d?.revision === "number") g.bestRevision = d.revision;
    }
  }
  if (g.bestWrite !== 0) {
    const r = og.platform.result(world, g.bestWrite);
    if (r !== null) {
      g.bestWrite = 0;
      if (r.ok) g.bestRevision = (r.data as { revision: number }).revision;
    }
  }
}

/** The play is over: the results, the best kept, the score on the board, the telemetry. */
function finish(world: World, g: GameV): void {
  g.phase = Phase.Results;
  g.stopBand = true;
  if (g.cleared && g.score > g.best) {
    g.best = g.score;
    g.newBest = true;
    const w = og.platform.storeSet(world, { key: "best", value: g.score, revision: g.bestRevision });
    if (w.ok) g.bestWrite = w.value;
  }
  if (g.cleared) must2(og.platform.leaderboardSubmit(world, { board: "neon-pulse", score: g.score }));
  must(og.platform.telemetry(world, "song.over", { cleared: g.cleared, score: g.score, accuracy: accuracy(g), maxCombo: g.maxCombo, miss: g.miss }));
}

function must2(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

/** Accuracy, 0 to 100: a Perfect counts whole, a Great 70 %, a Good 40 %, a Miss nothing. */
export function accuracy(g: GameV): number {
  if (g.judged === 0) return 100;
  return Math.round((1000 * (g.perfect + 0.7 * g.great + 0.4 * g.good)) / g.judged) / 10;
}

/** The grade of a cleared song by its accuracy. */
export function grade(acc: number): string {
  return acc >= 95 ? "S" : acc >= 85 ? "A" : acc >= 70 ? "B" : acc >= 50 ? "C" : "D";
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.sm, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "capsule", background: { token: "surface" } }, build);
}

function hud(world: World, g: GameV): void {
  const t = lazyTuning(world);
  const vp = og.ui.frameViewport();
  const phone = vp.width < vp.height;
  // The pad and the arrows play the lanes; they move the focus only on the pause menu.
  og.ui.setNavigation(g.paused || g.settingsOpen);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    capsule("Score", () => {
      og.ui.label("Label", say(world, "hud.score"), { step: "caption", color: { token: "fgMuted" }, nowrap: true });
      og.ui.counter("Value", g.score, { step: "xl", font: "display", durationMs: 300, width: px(phone ? 104 : 130) });
    });
    capsule("Groove", () => {
      og.ui.label("Label", say(world, "hud.groove"), { step: "caption", color: { token: "fgMuted" }, nowrap: true });
      og.ui.meter("Bar", g.groove, 1, { kind: "health", width: px(phone ? 90 : 130) });
    });
  });
  let paused = false;
  og.ui.anchor("Corner", { anchor: "topRight" }, () => {
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) paused = true;
  });
  if (paused) pause(world, g);
  // The combo beside the highway (above it on a phone): the count and its multiplier's progress.
  const multiplier = Math.min(4, 1 + Math.floor(g.combo / t.comboStep));
  og.ui.anchor("Combo", { anchor: phone ? "top" : "right", offsetY: phone ? 64 : 40, offsetX: phone ? 0 : -48, direction: "column", cross: "center", gap: S.xxs }, () => {
    og.ui.counter("Count", g.combo, { step: "displaySm", font: "display", durationMs: 120, align: "center", width: px(96) });
    og.ui.row("Head", { gap: S.xs, cross: "center" }, () => {
      og.ui.label("Label", say(world, "hud.combo"), { step: "caption", color: { token: "fgMuted" }, nowrap: true });
      og.ui.label("Multiplier", say(world, "hud.multiplier", { n: multiplier }), { step: "caption", color: { token: "accent" }, nowrap: true });
    });
    og.ui.meter("Next", multiplier >= 4 ? 1 : (g.combo % t.comboStep) / t.comboStep, 1, { width: px(96) });
  });
}

function settings(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  let cal = false;
  const before = JSON.stringify(prefsValue(p));
  const m = og.ui.modal("Settings", g.settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(440, og.ui.frameViewport().width - 32)) }, () => {
    const slider = (id: string, key: string, value: number, min = 0, max = 1, step = 0.05, shown = (v: number) => `${Math.round(v * 100)}`): number => {
      let out = value;
      og.ui.row(id, { width: grow, gap: S.md, cross: "center" }, () => {
        og.ui.label("Label", say(world, key), { width: px(110), nowrap: true });
        out = og.ui.slider("Slider", value, { min, max, step }).value;
        og.ui.label("Value", shown(out), { width: px(64), align: "end", font: "mono", nowrap: true });
      });
      return out;
    };
    p.master = slider("Master", "settings.master", p.master);
    p.music = slider("Music", "settings.music", p.music);
    p.effects = slider("Effects", "settings.effects", p.effects);
    p.offsetMs = slider("Offset", "settings.offset", p.offsetMs, -200, 200, 5, (v) => say(world, "settings.ms", { n: Math.round(v) }));
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "fgMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), p.language, { horizontal: true });
    if (r.selected !== p.language) setLanguage(world, p, r.selected);
    p.reducedMotion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, gap: S.sm, align: "end" }, () => {
      if (g.phase === Phase.Title && og.ui.button("Calibrate", say(world, "settings.calibrate")).clicked) cal = true;
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue(p)) !== before) applyPrefs(world, p);
  if (g.settingsOpen && (done || cal || m.dismissed)) {
    g.settingsOpen = false;
    saveSettings(world, p);
    if (cal) calibrate(world, g);
  }
}

/** The calibration: tap on the clicks; the mean error of eight taps is the offset. */
function calibration(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  let again = false;
  const measured = g.calTaps > 0 ? g.calSum / g.calTaps : p.offsetMs;
  const m = og.ui.modal("Calibration", true, { title: say(world, "cal.title"), width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.label("Body", say(world, "cal.body"), { width: grow });
    og.ui.row("Taps", { width: grow, gap: S.md, cross: "center" }, () => {
      og.ui.label("Count", say(world, "cal.taps", { n: g.calTaps }), { nowrap: true, width: grow });
      og.ui.label("Offset", say(world, "cal.offset", { n: Math.round(measured) }), { font: "mono", nowrap: true, color: { token: "accent" } });
    });
    og.ui.meter("Progress", g.calTaps, 8, { width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm, align: "end" }, () => {
      if (og.ui.button("Again", say(world, "cal.reset")).clicked) again = true;
      if (og.ui.button("Done", say(world, "cal.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (again) {
    g.calTaps = 0;
    g.calSum = 0;
  }
  if (done || m.dismissed) {
    if (done && g.calTaps >= 4) {
      p.offsetMs = Math.round(measured);
      saveSettings(world, p);
    }
    g.phase = Phase.Title;
    g.stopBand = true;
  }
}

/** The results: cleared (with a grade) or failed, the counts, the best. */
function results(world: World, g: GameV): "again" | "title" | null {
  let chose: "again" | "title" | null = null;
  const acc = accuracy(g);
  og.ui.modal("Results", true, { title: say(world, g.cleared ? "results.cleared" : "results.failed"), locked: true, width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center", gap: S.md, cross: "center" }, () => {
      og.ui.transition("Grade", true, { kind: "pop" }, () => og.ui.label("Letter", g.cleared ? grade(acc) : "–", { step: "displayMd", font: "display", color: { token: g.cleared ? "accent" : "fgMuted" } }));
      if (g.newBest) og.ui.label("NewBest", say(world, "results.newBest"), { color: { token: "accent" }, nowrap: true });
    });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      og.ui.keyValue("Score", say(world, "results.score"), `${g.score}`, true);
      og.ui.keyValue("Accuracy", say(world, "results.accuracy"), say(world, "results.percent", { n: acc }), true);
      og.ui.keyValue("Combo", say(world, "results.combo"), `${g.maxCombo}`, true);
      og.ui.keyValue("Perfect", say(world, "results.perfect"), `${g.perfect}`, true);
      og.ui.keyValue("Great", say(world, "results.great"), `${g.great}`, true);
      og.ui.keyValue("Good", say(world, "results.good"), `${g.good}`, true);
      og.ui.keyValue("Miss", say(world, "results.miss"), `${g.miss}`, true);
    });
    og.ui.label("Best", say(world, "results.best", { n: g.best }), { color: { token: "fgMuted" }, align: "center", width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm }, () => {
      if (og.ui.button("Again", say(world, "results.again"), { variant: "primary", width: grow }).clicked) chose = "again";
      if (og.ui.button("Title", say(world, "results.title"), { width: grow }).clicked) chose = "title";
    });
  });
  return chose;
}

/** The screens, the layout for the surface, the buses, the platform's answers, the play's start, pause and end. */
export const Ui = defineSystem("Ui", { reads: [Tuning, og.Input, og.Gamepad], writes: [Game, Prefs, og.Camera, og.Audio] }, (_q, time, world) => {
  const game = world.resource(Game);
  const prefs = world.resource(Prefs);
  if (!game.ok || !prefs.ok) return;
  const g = game.value;
  const p = prefs.value;
  platform(world, g, p);
  if (time.tick === 0) applyPrefs(world, p);
  // The layout: 10 m tall and three 1.5 m lanes on a wide screen; 7.2 m across a portrait one with 2 m lanes.
  const vp = og.ui.frameViewport();
  if (vp.width > 0 && (vp.width !== g.viewW || vp.height !== g.viewH)) {
    g.viewW = vp.width;
    g.viewH = vp.height;
    const portrait = vp.width < vp.height;
    const height = portrait ? (7.2 * vp.height) / vp.width : 10;
    const cam = world.resource(og.Camera);
    if (cam.ok) cam.value.height = height;
    g.laneW = portrait ? 2 : 1.5;
    g.hitY = -height / 2 + (portrait ? 2.6 : 1.8);
    g.topY = height / 2 - (portrait ? 2.4 : 0.2);
  }
  const hold = g.phase === Phase.Play && (g.paused || g.settingsOpen);
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  const input = world.resource(og.Input);
  const pad = world.resource(og.Gamepad);
  if (g.phase === Phase.Play && !g.paused && input.ok && pad.ok && (input.value.pressed("Escape") || input.value.pressed("KeyP") || pad.value.pressed("start"))) pause(world, g);
  if (g.phase === Phase.Play && g.over) finish(world, g);

  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : g.phase === Phase.Results ? "results" : "calibrate";
  const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "wipe", durationMs: 500 }).shown;
  if (shown === "title") {
    og.ui.setNavigation(true);
    const r = og.ui.mainMenu("Title", {
      title: say(world, "title.name"),
      subtitle: say(world, "title.tagline"),
      items: [
        { id: "play", label: say(world, "title.play"), primary: true },
        { id: "calibrate", label: say(world, "title.calibrate") },
        { id: "settings", label: say(world, "title.settings") },
      ],
      footer: say(world, "title.footer"),
    });
    if (!g.settingsOpen && r.selected === "play") begin(world, g);
    if (!g.settingsOpen && r.selected === "calibrate") calibrate(world, g);
    if (r.selected === "settings") g.settingsOpen = true;
  } else if (shown === "play") {
    hud(world, g);
    const m = og.ui.pauseMenu("Pause", {
      open: g.paused && !g.settingsOpen,
      title: say(world, "pause.title"),
      items: [
        { id: "resume", label: say(world, "pause.resume"), primary: true },
        { id: "restart", label: say(world, "pause.restart") },
        { id: "settings", label: say(world, "pause.settings") },
        { id: "quit", label: say(world, "pause.quit") },
      ],
    });
    if (m.selected === "resume") resume(world, g);
    if (m.selected === "restart") begin(world, g);
    if (m.selected === "settings") g.settingsOpen = true;
    if (m.selected === "quit") {
      g.paused = false;
      g.pausedAt = -1;
      g.phase = Phase.Title;
      g.stopBand = true;
    }
  } else if (shown === "results") {
    og.ui.setNavigation(true);
    const c = results(world, g);
    if (c === "again") begin(world, g);
    if (c === "title") g.phase = Phase.Title;
  } else {
    og.ui.setNavigation(true);
    calibration(world, g, p);
  }
  settings(world, g, p);
  if (g.language !== p.language) g.language = p.language;
});
