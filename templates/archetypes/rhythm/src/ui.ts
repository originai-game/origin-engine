// Three-Lane Beat's screens on og.ui and the style in assets/styles/lanebeat
// (the engine's sci-fi pack): the title and the pause menu (the game's
// screen documents, assets/ui/main-menu.ogt and pause.ogt), the HUD (the
// score and the groove top left, Pause top right, the combo and its
// multiplier beside the highway), the settings with the offset, the calibration
// and the results. Screens change through a wipe (a fade with reduced
// motion). Every string is a key of assets/strings/*.ogt.
//
// The Ui system also starts a play (the count-in, then bar 1), pauses it
// (the band stops, the clock freezes) and resumes it on the beat it paused
// on after a new count-in (the stems continue part-way), ends it with the results, and
// keeps the settings and the best score in og.platform.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, Phase, Prefs, Rival, Tuning } from "./components.ts";
import { device, pressMenu } from "./device.ts";
import { ONLINE, SEATS } from "./seats.ts";
import { beatAt, beatLength, lazyTuning, must, seatPlays, worldTime } from "./systems.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "zh-CN", key: "lang.zh" },
] as const;

/** The tokens' spacing scale in CSS px (og.ui.frameTheme answers it too, but builds the whole theme each call). */
const S = { xxs: 2, xs: 4, sm: 8, md: 12, lg: 16, xl: 24 } as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;
/** A seat's presses and tally: Game's seat fields or Rival. */
type SeatV = Value<typeof Rival>;

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}

const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

/** A new play as of `now` (the tick Play was pressed on): every seat's counts cleared, the count-in from the next beat, then bar 1 (or `fromBar`). */
export function begin(world: World, g: GameV, fromBar = 1, now = worldTime(world)): void {
  const t = lazyTuning(world);
  const len = beatLength(g);
  g.phase = Phase.Play;
  g.paused = false;
  g.pausedAt = -1;
  for (const s of seatPlays(world)) {
    s.score = 0;
    s.combo = 0;
    s.maxCombo = 0;
    s.perfect = 0;
    s.great = 0;
    s.good = 0;
    s.miss = 0;
    s.hits = 0;
    s.judged = 0;
    s.taps = 0;
    s.groove = t.grooveStart;
    s.failed = false;
    s.lastJudgement = -1;
  }
  g.scheduled = 0;
  g.cleared = false;
  g.over = false;
  g.endAt = 0;
  g.newBest = false;
  g.countShown = -1;
  g.resetNotes = true;
  g.stopBand = true;
  // The first bar starts countInBeats after the next whole tick: the clicks lead in.
  const from = (Math.max(1, Math.min(g.bars, fromBar)) - 1) * 4;
  g.fromBeat = from;
  g.countTo = Math.ceil((now + 0.25) * 60) / 60 + t.countInBeats * len;
  g.songStart = g.countTo - from * len;
  g.bandFrom = from / 4 + 1;
  g.bandBeat = from;
  g.scheduleUpTo = from + t.scrollBeats + 1;
}

/** Pause as of `now` (the tick it was pressed on): the clock freezes there, the band stops. */
export function pause(g: GameV, now: number): void {
  if (g.phase !== Phase.Play || g.paused || g.over) return;
  g.paused = true;
  g.pausedAt = now;
  g.stopBand = true;
}

/** Resume on the beat it was paused on, after a new count-in from `now`: the stems sounding then continue from where they were (AudioSource.startOffset). */
export function resume(world: World, g: GameV, now = worldTime(world)): void {
  if (!g.paused) return;
  const t = lazyTuning(world);
  const len = beatLength(g);
  const from = Math.max(0, beatAt(g, g.pausedAt));
  const countTo = Math.ceil((now + 0.25) * 60) / 60 + t.countInBeats * len;
  g.songStart = countTo - from * len;
  g.countTo = countTo;
  g.bandFrom = Math.floor(from / 4) + 1;
  g.bandBeat = from;
  g.scheduleUpTo = Math.ceil(from) + t.scrollBeats + 1;
  g.countShown = -1;
  g.paused = false;
  g.pausedAt = -1;
}

/** Calibration: the clock clicks every beat from the next one after `now`; taps measure the offset. */
export function calibrate(world: World, g: GameV, now = worldTime(world)): void {
  g.phase = Phase.Calibrate;
  g.calTaps = 0;
  g.calSum = 0;
  g.stopBand = true;
  g.songStart = Math.ceil((now + 0.5) * 60) / 60;
}

/** Back to the title (from the pause, the results or the calibration). */
export function toTitle(g: GameV): void {
  g.paused = false;
  g.pausedAt = -1;
  g.phase = Phase.Title;
  g.stopBand = true;
}

/** The settings modal is up on this device (the device's own, never the world's). */
let settingsOpen = false;

/** This device's calibrated offset; one-device play keeps its record in Prefs (a room's world keeps none). */
function setOffset(p: PrefsV, ms: number): void {
  device.offsetMs = ms;
  if (!ONLINE && p.offsetMs !== ms) p.offsetMs = ms;
}

/** The buses' volumes from the settings (og.Audio is each peer's own and in no snapshot, so a restored or joining world takes them again). */
function applyPrefs(world: World, p: PrefsV): void {
  const audio = world.resource(og.Audio);
  if (!audio.ok) return;
  const a = audio.value;
  if (a.master !== p.master) a.master = p.master;
  if (a.music !== p.music) a.music = p.music;
  if (a.sfx !== p.effects) a.sfx = p.effects;
  if (a.ui !== p.effects) a.ui = p.effects;
}

function setLanguage(world: World, p: PrefsV, index: number): void {
  p.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[p.language]!.code);
}

function prefsValue(p: PrefsV) {
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, language: LANGUAGES[p.language]!.code, reducedMotion: p.reducedMotion, offsetMs: device.offsetMs };
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
        if (typeof s.offsetMs === "number") setOffset(p, s.offsetMs);
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

/** The play is over: the results; alone the best kept, the score on the board; the telemetry. */
function finish(world: World, g: GameV): void {
  g.phase = Phase.Results;
  g.stopBand = true;
  const rival = SEATS > 1 ? must3(world.resource(Rival)) : null;
  if (rival === null && g.cleared && g.score > g.best) {
    g.best = g.score;
    g.newBest = true;
    const w = og.platform.storeSet(world, { key: "best", value: g.score, revision: g.bestRevision });
    if (w.ok) g.bestWrite = w.value;
  }
  if (rival === null && g.cleared) must2(og.platform.leaderboardSubmit(world, { board: "neon-pulse", score: g.score }));
  must(og.platform.telemetry(world, "song.over", { cleared: g.cleared, score: g.score, accuracy: accuracy(g), maxCombo: g.maxCombo, miss: g.miss, ...(rival ? { rivalScore: rival.score, rivalAccuracy: accuracy(rival) } : {}) }));
}

function must2(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

function must3<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Accuracy, 0 to 100: a Perfect counts whole, a Great 70 %, a Good 40 %, a Miss nothing. */
export function accuracy(s: SeatV): number {
  if (s.judged === 0) return 100;
  return Math.round((1000 * (s.perfect + 0.7 * s.great + 0.4 * s.good)) / s.judged) / 10;
}

/** The grade of a cleared song by its accuracy. */
export function grade(acc: number): string {
  return acc >= 95 ? "S" : acc >= 85 ? "A" : acc >= 70 ? "B" : acc >= 50 ? "C" : "D";
}

/** The seat with the higher score (-1 a draw). */
export function winner(plays: readonly SeatV[]): number {
  if (plays.length < 2 || plays[0]!.score === plays[1]!.score) return -1;
  return plays[0]!.score > plays[1]!.score ? 0 : 1;
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.sm, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

/** The seat this device plays in a room (-1 on one device). */
function ownSeat(world: World): number {
  if (!ONLINE) return -1;
  const lock = world.resource(og.Lockstep);
  return lock.ok && lock.value.running ? lock.value.localPlayer : -1;
}

function hud(world: World, g: GameV): void {
  const t = lazyTuning(world);
  const vp = og.ui.frameViewport();
  const phone = vp.width < vp.height;
  // The pad and the arrows play the lanes; they move the focus only on the pause menu.
  og.ui.setNavigation(g.paused || settingsOpen);
  const plays = seatPlays(world);
  const versus = plays.length > 1;
  const cam = world.resource(og.Camera);
  const pxPerM = cam.ok && cam.value.height > 0 ? vp.height / cam.value.height : 0;
  const mine = ownSeat(world);
  plays.forEach((s, seat) => {
    const id = (name: string) => (seat === 0 ? name : `${name}${seat}`);
    // Each seat's score and groove in its own top corner (P1 left, P2 right).
    og.ui.anchor(id("Status"), { anchor: seat === 0 ? "topLeft" : "topRight", direction: "column", gap: S.sm, cross: seat === 0 ? "start" : "end" }, () => {
      capsule("Score", () => {
        if (versus) og.ui.label("Seat", say(world, seat === mine ? "hud.you" : "hud.seat", { n: seat + 1 }), { step: "caption", color: { token: "primary" }, nowrap: true });
        og.ui.label("Label", say(world, "hud.score"), { step: "caption", color: { token: "inkMuted" }, nowrap: true });
        og.ui.counter("Value", s.score, { step: "xl", font: "display", durationMs: 300, width: px(versus && phone ? 60 : phone ? 104 : 130) });
      });
      capsule("Groove", () => {
        og.ui.label("Label", say(world, "hud.groove"), { step: "caption", color: { token: "inkMuted" }, nowrap: true });
        og.ui.meter("Bar", s.groove, 1, { kind: "health", width: px(versus && phone ? 64 : phone ? 90 : 130) });
      });
    });
    // The combo beside its highway (above it on a phone): the count and its multiplier's progress.
    const multiplier = Math.min(4, 1 + Math.floor(s.combo / t.comboStep));
    const side = seat === 0 ? -1 : 1;
    const at = !versus
      ? { anchor: phone ? "top" : "right", offsetY: phone ? 64 : 40, offsetX: phone ? 0 : -48 }
      : phone
        ? { anchor: "top", offsetY: 150, offsetX: side * g.spread * pxPerM }
        : { anchor: seat === 0 ? "left" : "right", offsetY: 40, offsetX: -side * 48 };
    og.ui.anchor(id("Combo"), { ...(at as { anchor: "top" }), direction: "column", cross: "center", gap: S.xxs }, () => {
      og.ui.counter("Count", s.combo, { step: "displaySm", font: "display", durationMs: 120, align: "center", width: px(96) });
      og.ui.row("Head", { gap: S.xs, cross: "center" }, () => {
        og.ui.label("Label", say(world, "hud.combo"), { step: "caption", color: { token: "inkMuted" }, nowrap: true });
        og.ui.label("Multiplier", say(world, "hud.multiplier", { n: multiplier }), { step: "caption", color: { token: "primary" }, nowrap: true });
      });
      og.ui.meter("Next", multiplier >= 4 ? 1 : (s.combo % t.comboStep) / t.comboStep, 1, { width: px(96) });
    });
  });
  // Pause top right alone, top centre between two seats' corners.
  og.ui.anchor("Corner", { anchor: versus ? "top" : "topRight" }, () => {
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) pressMenu(world, "pause");
  });
}

function settings(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  let cal = false;
  const before = JSON.stringify(prefsValue(p));
  const m = og.ui.modal("Settings", settingsOpen, { title: say(world, "settings.title"), width: px(Math.min(440, og.ui.frameViewport().width - 32)) }, () => {
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
    setOffset(p, slider("Offset", "settings.offset", device.offsetMs, -200, 200, 5, (v) => say(world, "settings.ms", { n: Math.round(v) })));
    og.ui.label("LanguageLabel", say(world, "settings.language"), { color: { token: "inkMuted" } });
    const r = og.ui.radio("Language", LANGUAGES.map((l) => say(world, l.key)), p.language, { horizontal: true });
    if (r.selected !== p.language) setLanguage(world, p, r.selected);
    p.reducedMotion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
    og.ui.row("Footer", { width: grow, gap: S.sm, align: "end" }, () => {
      if (g.phase === Phase.Title && !ONLINE && og.ui.button("Calibrate", say(world, "settings.calibrate")).clicked) cal = true;
      if (og.ui.button("Done", say(world, "settings.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (JSON.stringify(prefsValue(p)) !== before) applyPrefs(world, p);
  if (settingsOpen && (done || cal || m.dismissed)) {
    settingsOpen = false;
    saveSettings(world, p);
    if (cal) pressMenu(world, "calibrate");
  }
}

/** The calibration: tap on the clicks; this device's offset plus the mean error of eight taps is the offset to keep. */
function calibration(world: World, g: GameV, p: PrefsV): void {
  let done = false;
  let again = false;
  const measured = g.calTaps > 0 ? device.offsetMs + g.calSum / g.calTaps : device.offsetMs;
  const m = og.ui.modal("Calibration", true, { title: say(world, "cal.title"), width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.label("Body", say(world, "cal.body"), { width: grow });
    og.ui.row("Taps", { width: grow, gap: S.md, cross: "center" }, () => {
      og.ui.label("Count", say(world, "cal.taps", { n: g.calTaps }), { nowrap: true, width: grow });
      og.ui.label("Offset", say(world, "cal.offset", { n: Math.round(measured) }), { font: "mono", nowrap: true, color: { token: "primary" } });
    });
    og.ui.meter("Progress", g.calTaps, 8, { width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm, align: "end" }, () => {
      if (og.ui.button("Again", say(world, "cal.reset")).clicked) again = true;
      if (og.ui.button("Done", say(world, "cal.done"), { variant: "primary" }).clicked) done = true;
    });
  });
  if (again) pressMenu(world, "calibrate");
  if (done || m.dismissed) {
    if (done && g.calTaps >= 4) {
      setOffset(p, Math.round(measured));
      saveSettings(world, p);
    }
    pressMenu(world, "title");
  }
}

/** The results: cleared (with a grade) or failed, the counts, the best; two seats' side by side, the higher score named. */
function results(world: World, g: GameV): "again" | "title" | null {
  let chose: "again" | "title" | null = null;
  const plays = seatPlays(world);
  const versus = plays.length > 1;
  const win = winner(plays);
  const lead = plays[Math.max(0, win)]!;
  const acc = accuracy(lead);
  const both = (f: (s: SeatV) => string | number) => plays.map(f).join("  ·  ");
  og.ui.modal("Results", true, { title: say(world, g.cleared ? "results.cleared" : "results.failed"), locked: true, width: px(Math.min(420, og.ui.frameViewport().width - 32)) }, () => {
    og.ui.row("Mark", { width: grow, align: "center", gap: S.md, cross: "center" }, () => {
      og.ui.transition("Grade", true, { kind: "pop" }, () => og.ui.label("Letter", g.cleared ? grade(acc) : "–", { step: "displayMd", font: "display", color: { token: g.cleared ? "primary" : "inkMuted" } }));
      if (versus) og.ui.label("Winner", win < 0 ? say(world, "results.draw") : say(world, "results.wins", { n: win + 1 }), { step: "xl", font: "display", color: { token: "primary" }, nowrap: true });
      if (g.newBest) og.ui.label("NewBest", say(world, "results.newBest"), { color: { token: "primary" }, nowrap: true });
    });
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      if (versus) og.ui.keyValue("Seats", "", both((s) => say(world, "hud.seat", { n: plays.indexOf(s) + 1 })), true);
      og.ui.keyValue("Score", say(world, "results.score"), both((s) => s.score), true);
      og.ui.keyValue("Accuracy", say(world, "results.accuracy"), both((s) => say(world, "results.percent", { n: accuracy(s) })), true);
      og.ui.keyValue("Combo", say(world, "results.combo"), both((s) => s.maxCombo), true);
      og.ui.keyValue("Perfect", say(world, "results.perfect"), both((s) => s.perfect), true);
      og.ui.keyValue("Great", say(world, "results.great"), both((s) => s.great), true);
      og.ui.keyValue("Good", say(world, "results.good"), both((s) => s.good), true);
      og.ui.keyValue("Miss", say(world, "results.miss"), both((s) => s.miss), true);
    });
    if (!versus) og.ui.label("Best", say(world, "results.best", { n: g.best }), { color: { token: "inkMuted" }, align: "center", width: grow });
    og.ui.row("Actions", { width: grow, gap: S.sm }, () => {
      if (og.ui.button("Again", say(world, "results.again"), { variant: "primary", width: grow }).clicked) chose = "again";
      if (og.ui.button("Title", say(world, "results.title"), { width: grow }).clicked) chose = "title";
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

/**
 * The screens, the layout for the surface, the buses, the platform's
 * answers and the results. What changes the play (Play, Pause, Resume,
 * Restart, the title, the calibration) is pressed for the seat (press():
 * src/device.ts's pressMenu) and acted on by Controls, so a room's peers
 * all act on it on one tick.
 */
export const Ui = defineSystem("Ui", { reads: [Tuning, Rival, og.Lockstep], writes: [Game, Prefs, og.Camera, og.Audio] }, (_q, _time, world) => {
  const game = world.resource(Game);
  const prefs = world.resource(Prefs);
  if (!game.ok || !prefs.ok) return;
  const g = game.value;
  const p = prefs.value;
  platform(world, g, p);
  applyPrefs(world, p);
  device.latencyMs = og.audio.outputLatency(world) * 1000;
  // The layout: 10 m tall and three 1.5 m lanes on a wide screen; 7.2 m across a portrait one with 2 m lanes
  // (two seats: their highways side by side, 1.1 m lanes on a portrait one).
  const vp = og.ui.frameViewport();
  if (vp.width > 0 && (vp.width !== g.viewW || vp.height !== g.viewH)) {
    g.viewW = vp.width;
    g.viewH = vp.height;
    const portrait = vp.width < vp.height;
    const height = portrait ? (7.2 * vp.height) / vp.width : 10;
    const cam = world.resource(og.Camera);
    if (cam.ok) cam.value.height = height;
    g.laneW = portrait ? (SEATS > 1 ? 1.1 : 2) : 1.5;
    g.spread = SEATS > 1 ? (portrait ? 1.8 : 3.75) : 0;
    g.hitY = -height / 2 + (portrait ? 2.6 : 1.8);
    g.topY = height / 2 - (portrait ? 2.4 : 0.2);
    device.layout = { laneW: g.laneW, spread: g.spread, hitY: g.hitY };
  }
  const hold = g.phase === Phase.Play && g.paused;
  if (hold !== g.held) {
    must(og.time.setScale(world, hold ? 0 : 1));
    g.held = hold;
  }
  if (g.phase === Phase.Play && g.over) finish(world, g);

  const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : g.phase === Phase.Results ? "results" : "calibrate";
  const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "wipe", durationMs: 500 }).shown;
  // The title and the pause are the game's screen documents (assets/ui/main-menu.ogt, pause.ogt).
  // Calibration is this device's own and off in a room (src/device.ts): no Calibrate there.
  for (const e of layer(world, "main-menu", shown === "title", { calibrate: !ONLINE })) {
    if (!settingsOpen && e === "play") pressMenu(world, "start");
    if (!settingsOpen && e === "calibrate" && !ONLINE) pressMenu(world, "calibrate");
    if (e === "settings") settingsOpen = true;
  }
  // Back from its Settings the pause keeps the focus on Settings, as the pause shipped.
  const fromSettings = og.ui.memory("pause.fromSettings", 0);
  const pauseUp = shown === "play" && g.paused && !settingsOpen;
  for (const e of layer(world, "pause", pauseUp)) {
    if (e === "resume") pressMenu(world, "resume");
    if (e === "restart") pressMenu(world, "start");
    if (e === "settings") {
      settingsOpen = true;
      og.ui.remember("pause.fromSettings", 1);
    }
    if (e === "quit") pressMenu(world, "title");
  }
  if (pauseUp && fromSettings === 1) {
    og.ui.setFocus("pause/Pause/Panel/Menu/Settings");
    og.ui.remember("pause.fromSettings", 0);
  }
  if (shown === "title") {
    og.ui.setNavigation(true);
  } else if (shown === "play") {
    hud(world, g);
  } else if (shown === "results") {
    og.ui.setNavigation(true);
    const c = results(world, g);
    if (c === "again") pressMenu(world, "start");
    if (c === "title") pressMenu(world, "title");
  } else {
    og.ui.setNavigation(true);
    calibration(world, g, p);
  }
  settings(world, g, p);
  if (g.language !== p.language) g.language = p.language;
});
