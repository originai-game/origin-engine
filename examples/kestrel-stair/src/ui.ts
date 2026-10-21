// The Kestrel Stair's screens, on og.ui with the paper skin
// (assets/skins/kestrel, brought by the look):
//
// - the title (og.ui.mainMenu): Continue with the save slot's place and
//   hour, New journey, Chapters (og.ui.levelSelect: the four chapters, their
//   stars, the ones not yet reached locked), Records (og.ui.leaderboardScreen:
//   the fastest ascents and the courier's neighbours, loading, error and
//   empty states honest), Settings (og.ui.settingsScreen, every word from the
//   string tables);
// - the intro and the ending (og.ui.cutscene), the tutorial on a first
//   journey (og.ui.tutorialOverlay, a spotlight on each part of the HUD);
// - the HUD: strength and sigils top left, the hour and the bag and pause
//   buttons top right, the prompt for what F would use at the bottom (it
//   shakes when F is refused);
// - the bag (og.ui.inventoryScreen: the letter, the sigils, the hillwort;
//   items dragged between slots, hillwort eaten with Use), the pause menu,
//   the results (og.ui.resultsScreen, under the summit look after a won
//   climb), toasts; a wipe between screens (a fade with reduced motion).
//
// The Ui system also runs the clock, the hold (time scale 0 under a menu),
// the run's start and end, the looks, and og.platform's answers
// (src/save.ts).

import { og, defineSystem } from "@origingame/engine";
import type { GameSettings, LevelInfo, Look, Sound as SoundHandle, Value, World } from "@origingame/engine";
import { Ending, Game, Hero, HOURS, MAX_HP, Near, OneByOne, Phase, Prefs, Scene, Sound, Tuning } from "./components.ts";
import { placeHero } from "./hero.ts";
import { CHAPTERS, chapterAt, WAYSTONES } from "./layout.ts";
import { eatHerb } from "./pickups.ts";
import { afterRestore, answers, continueRun, readAll, readRecords, submitAscent, writeProgress } from "./save.ts";
import { uiSounds } from "./sound.ts";
import type { UiSounds } from "./sound.ts";
import { say } from "./text.ts";
import { begin, flush } from "./state.ts";
import { must, setHour } from "./world.ts";

export const LANGUAGES = [
  { code: "en", key: "lang.en" },
  { code: "ja", key: "lang.ja" },
] as const;

/** Ticks between a run's end and its results: the moment plays out first. */
export const OVER_DELAY = 90;

const S = { xs: 4, sm: 8, md: 12, lg: 16 } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

/** The bag's slots: item ids by slot, kept as text in Game.bag. */
const BAG_SLOTS = 12;
const ITEMS = ["letter", "sigil0", "sigil1", "sigil2", "herb"] as const;

/** The UI's sounds and the player's motion and contrast: og.ui.configure keeps the look's skin. */
export function configureUi(world: World, p: PrefsV): void {
  const sounds: UiSounds = uiSounds(world);
  must(og.ui.configure(world, { reducedMotion: p.reducedMotion, contrast: p.highContrast ? "high" : "standard", sounds, toastAnchor: "top" }));
}

function applyPrefs(world: World, p: PrefsV): void {
  const audio = must(world.resource(og.Audio));
  const m = p.muted ? 0 : p.master;
  audio.master = m;
  audio.music = p.music;
  audio.sfx = p.effects;
  audio.ui = p.effects;
  configureUi(world, p);
}

function setLanguage(world: World, p: PrefsV, index: number): void {
  p.language = Math.max(0, Math.min(LANGUAGES.length - 1, index));
  og.i18n.setLocale(world, LANGUAGES[p.language]!.code);
}

function settingsOf(p: PrefsV): GameSettings {
  return { masterVolume: p.master, musicVolume: p.music, sfxVolume: p.effects, muted: p.muted, reducedMotion: p.reducedMotion, highContrast: p.highContrast, sensitivity: p.sensitivity, invertY: p.invertY, language: LANGUAGES[p.language]!.code };
}

/** Settings from the platform or the settings screen, into Prefs. */
function takeSettings(world: World, p: PrefsV, s: Partial<Record<string, unknown>>): void {
  if (typeof s.masterVolume === "number") p.master = s.masterVolume;
  if (typeof s.musicVolume === "number") p.music = s.musicVolume;
  if (typeof s.sfxVolume === "number") p.effects = s.sfxVolume;
  if (typeof s.muted === "boolean") p.muted = s.muted;
  if (typeof s.reducedMotion === "boolean") p.reducedMotion = s.reducedMotion;
  if (typeof s.highContrast === "boolean") p.highContrast = s.highContrast;
  if (typeof s.sensitivity === "number") p.sensitivity = s.sensitivity;
  if (typeof s.invertY === "boolean") p.invertY = s.invertY;
  const at = LANGUAGES.findIndex((l) => l.code === s.language);
  if (at >= 0 && at !== p.language) setLanguage(world, p, at);
  applyPrefs(world, p);
}

function clockText(hour: number): string {
  const h = Math.floor(hour);
  const m = Math.floor((hour - h) * 60);
  return `${h}:${m < 10 ? "0" : ""}${m}`;
}

function seconds(s: number): string {
  const m = Math.floor(s / 60);
  const r = Math.floor(s % 60);
  return `${m}:${r < 10 ? "0" : ""}${r}`;
}

/** A new run from chapter `c`: the courier at its waystone, strength full, the clock at its hour. */
export function startRun(world: World, g: GameV, c: number): void {
  const ch = CHAPTERS[c]!;
  const w = WAYSTONES[ch.waystone]!;
  g.phase = Phase.Play;
  g.ending = Ending.None;
  g.chapter = c;
  g.paused = false;
  g.bagOpen = false;
  g.chaptersOpen = false;
  g.recordsOpen = false;
  g.hp = MAX_HP;
  g.herbs = 0;
  g.herbsUsed = 0;
  g.kills = 0;
  g.hits = 0;
  g.hurts = 0;
  g.swings = 0;
  g.jumps = 0;
  g.landings = 0;
  g.steps = 0;
  g.time = 0;
  g.reached = 1 << c;
  g.rested = 0;
  g.overTicks = 0;
  g.newBest = false;
  g.invulnerable = 0;
  g.knock = 0;
  g.attack = 0;
  g.score = 0;
  g.rank = -1;
  g.hour = HOURS.start + c * 0.35;
  g.start = true;
  g.bag = JSON.stringify(["letter", "sigil0", "sigil1", "sigil2", "herb", "", "", "", "", "", "", ""]);
  // A journey from the start opens with the abbess's words.
  g.scene = c === 0 ? Scene.Intro : Scene.None;
  g.line = 0;
  g.tutorial = false;
  g.tutorialStep = 0;
  placeHero(world, g, w.x - 1.2, w.y, w.z - 0.8, Math.PI);
  setHour(world, g.hour);
}

/** Back to the title: the stair reset behind it. */
export function toTitle(world: World, g: GameV): void {
  g.phase = Phase.Title;
  g.ending = Ending.None;
  g.paused = false;
  g.bagOpen = false;
  g.scene = Scene.None;
  g.tutorial = false;
  g.start = true;
  g.hour = HOURS.title;
  setHour(world, g.hour);
}

/** The run's score: daylight left, foes and sigils. */
export function score(g: GameV, t: Value<typeof Tuning>): number {
  const left = Math.max(0, t.duskSeconds - g.time);
  const sigils = [0, 1, 2].filter((b) => (g.sigils & (1 << b)) !== 0).length;
  return Math.round((g.ending === Ending.Rung ? left * 10 : 0) + g.kills * 150 + sigils * 200);
}

/** Stars for the run: one for the bell, one for under three and a half minutes, one for losing under a third of strength. */
function stars(g: GameV): number {
  if (g.ending !== Ending.Rung) return 0;
  return 1 + (g.time < 210 ? 1 : 0) + (g.hurts <= 3 ? 1 : 0);
}

/** The clock, the endings, the chapters reached. */
function clock(world: World, g: GameV, t: Value<typeof Tuning>, dt: number): void {
  if (g.phase !== Phase.Play && g.phase !== Phase.Ending) return;
  if (g.phase === Phase.Play && g.ending === Ending.None && !g.held) {
    g.time += dt;
    g.hour = Math.min(HOURS.night, HOURS.start + g.chapter * 0.35 + (g.time / t.duskSeconds) * (HOURS.night - HOURS.start));
    const c = chapterAt(g.heroX, g.heroY, g.heroZ);
    if ((g.reached & (1 << c)) === 0) {
      g.reached |= 1 << c;
      g.chaptersReached |= 1 << c;
    }
    if (g.hp <= 0) {
      g.ending = Ending.Fallen;
      g.sfx |= Sound.Lose;
    } else if (g.hour >= HOURS.night) {
      g.ending = Ending.Night;
      g.sfx |= Sound.Lose;
    }
    setHour(world, g.hour);
  }
  if (g.ending === Ending.None || g.phase !== Phase.Play) return;
  g.overTicks += 1;
  if (g.overTicks >= OVER_DELAY) finish(world, g, t);
}

/** The results: the best and the stars kept, a won ascent on the board. */
function finish(world: World, g: GameV, t: Value<typeof Tuning>): void {
  g.phase = Phase.Over;
  g.scene = Scene.None;
  g.score = score(g, t);
  if (g.score > g.bestScore) g.bestScore = g.score;
  if (g.ending === Ending.Rung) {
    if (g.chapter === 0 && (g.best === 0 || g.time < g.best)) {
      g.newBest = true;
      g.best = Math.round(g.time * 10) / 10;
    }
    if (g.chapter === 0) submitAscent(world, g);
  }
  // Each chapter reached this run keeps its best stars.
  const s = stars(g);
  let packed = g.chapterStars;
  for (let c = 0; c < 4; c++) {
    if ((g.reached & (1 << c)) === 0) continue;
    const have = (packed >> (c * 2)) & 3;
    const earned = g.ending === Ending.Rung ? s : 1;
    if (earned > have) packed = (packed & ~(3 << (c * 2))) | (earned << (c * 2));
  }
  g.chapterStars = packed;
  writeProgress(world, g);
}

/** The summit look for a won climb's end and results, the stair's everywhere else. */
function looks(world: World, g: GameV): void {
  const want = (g.phase === Phase.Ending || g.phase === Phase.Over) && g.ending === Ending.Rung ? 1 : 0;
  if (want === g.look) return;
  must(og.render.applyLook(world, (want === 1 ? g.lookSummit : g.lookStair) as unknown as Look));
  g.look = want;
}

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "capsule", background: { token: "surface" } }, build);
}

function hud(world: World, g: GameV): { bag: boolean; pause: boolean } {
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    capsule("Health", () => {
      og.ui.icon("Heart", "heart", { size: 22, tint: { token: g.hp < 30 ? "danger" : "accent" } });
      og.ui.meter("Strength", g.hp, MAX_HP, { kind: "health", width: px(150) });
    });
    capsule("Sigils", () => {
      for (let i = 0; i < 3; i++) og.ui.icon(`Sigil${i}`, "gem", { size: 20, tint: { token: (g.sigils & (1 << i)) !== 0 ? "accent" : "fillStrong" } });
      og.ui.label("Count", say(world, "hud.sigils", { n: [0, 1, 2].filter((b) => (g.sigils & (1 << b)) !== 0).length }), { font: "mono", nowrap: true });
    });
  });
  let bag = false;
  let pause = false;
  og.ui.anchor("Dusk", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    capsule("Clock", () => {
      const left = 1 - (g.hour - HOURS.start) / (HOURS.night - HOURS.start);
      og.ui.icon("Sun", "clock", { size: 20, tint: { token: left < 0.2 ? "danger" : "fg" } });
      og.ui.label("Hour", clockText(g.hour), { font: "mono", nowrap: true, width: px(48) });
      og.ui.meter("Daylight", Math.max(0, left), 1, { kind: "accent", width: px(90) });
    });
    if (og.ui.actionButton("Bag", say(world, "hud.bag"), { icon: "bag", size: 48 }).clicked) bag = true;
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 48 }).clicked) pause = true;
  });
  // The prompt pops in and out (og.ui.transition): what F would use now, or (leaving) what it was.
  const prompt = promptText(world, g, g.nearKind);
  if (prompt !== "") g.promptKind = g.nearKind;
  const shownPrompt = prompt !== "" ? prompt : promptText(world, g, g.promptKind);
  og.ui.anchor("Prompt", { anchor: "bottom", offsetY: -28, direction: "column", cross: "center" }, () => {
    og.ui.transition("Hint", prompt !== "", { kind: "pop" }, () => {
      capsule("Capsule", () => {
        og.ui.icon("Key", g.nearKind === Near.Gate && (g.sigils & 7) !== 7 ? "lock" : "flag", { size: 18, tint: { token: "accent" } });
        og.ui.label("Text", shownPrompt, { nowrap: true });
      });
      // A refused F shakes the prompt with the error sound (og-ui §6), never the camera.
      if (g.refusedAt !== 0 && world.tick - g.refusedAt <= 1) og.ui.shake({ sound: false });
    });
  });
  return { bag, pause };
}

function promptText(world: World, g: GameV, kind: number): string {
  switch (kind) {
    case Near.Sigil:
      return say(world, "prompt.sigil");
    case Near.Herb:
      return say(world, "prompt.herb");
    case Near.Waystone:
      return say(world, "prompt.waystone");
    case Near.Gate:
      return (g.sigils & 7) === 7 ? say(world, "prompt.gate") : say(world, "prompt.gateShut", { n: [0, 1, 2].filter((b) => (g.sigils & (1 << b)) !== 0).length });
    case Near.Bell:
      return g.gateOpen ? say(world, "prompt.bell") : "";
    default:
      return "";
  }
}

/** The bag: the letter, the sigils carried, the hillwort; drag to move, Use to eat. */
function bag(world: World, g: GameV): void {
  let slots: string[];
  try {
    slots = JSON.parse(g.bag) as string[];
  } catch {
    slots = [];
  }
  while (slots.length < BAG_SLOTS) slots.push("");
  const items = slots
    .map((id, slot) => ({ id, slot }))
    .filter(({ id }) => id === "letter" || (id.startsWith("sigil") && (g.sigils & (1 << Number(id.slice(5)))) !== 0) || (id === "herb" && g.herbs > 0))
    .map(({ id, slot }) => ({
      slot,
      id,
      name: say(world, `item.${id}`),
      description: say(world, `item.${id}.about`),
      icon: id === "letter" ? "map" : id === "herb" ? "leaf" : "gem",
      count: id === "herb" ? g.herbs : 0,
    }));
  const r = og.ui.inventoryScreen("Bag", { title: say(world, "bag.title"), slots: BAG_SLOTS, columns: 6, items, useLabel: say(world, "bag.use"), hintLabel: say(world, "bag.hint"), countLabel: say(world, "bag.count") });
  if (r.moved) {
    const a = slots[r.moved.from] ?? "";
    slots[r.moved.from] = slots[r.moved.to] ?? "";
    slots[r.moved.to] = a;
    g.bag = JSON.stringify(slots);
    g.bagMoves += 1;
  }
  if (r.used === "herb") eatHerb(world, g);
  if (r.closed) g.bagOpen = false;
  void ITEMS;
}

function intro(world: World): { speaker: string; text: string; portrait: string }[] {
  const a = say(world, "intro.abbess");
  const c = say(world, "intro.courier");
  return [
    { speaker: a, text: say(world, "intro.1"), portrait: "crown" },
    { speaker: a, text: say(world, "intro.2"), portrait: "crown" },
    { speaker: c, text: say(world, "intro.3"), portrait: "person" },
    { speaker: a, text: say(world, "intro.4"), portrait: "crown" },
  ];
}

function ending(world: World): { speaker: string; text: string; portrait: string }[] {
  const k = say(world, "end.keeper");
  const c = say(world, "intro.courier");
  return [
    { speaker: k, text: say(world, "end.1"), portrait: "music" },
    { speaker: c, text: say(world, "end.2"), portrait: "person" },
    { speaker: k, text: say(world, "end.3"), portrait: "music" },
  ];
}

function tutorialSteps(world: World) {
  return [
    { target: "Health", title: say(world, "tut.health.title"), text: say(world, "tut.health.text") },
    { target: "Sigils", title: say(world, "tut.sigils.title"), text: say(world, "tut.sigils.text") },
    { target: "Clock", title: say(world, "tut.clock.title"), text: say(world, "tut.clock.text") },
    { target: "Bag", title: say(world, "tut.bag.title"), text: say(world, "tut.bag.text") },
  ];
}

function chapters(world: World, g: GameV): void {
  // A card draws its name in display type on a 104 px square, and `columns` is not applied
  // (FINDINGS #15): the chapters are named by numerals there.
  const levels: LevelInfo[] = CHAPTERS.map((c, i) => ({
    name: say(world, `chapter.${c.key}`),
    stars: (g.chapterStars >> (i * 2)) & 3,
    locked: (g.chaptersReached & (1 << i)) === 0,
    best: i === 0 && g.best > 0 ? seconds(g.best) : "",
  }));
  const r = og.ui.levelSelect("Chapters", { title: say(world, "chapters.title"), levels, columns: 2 });
  if (r.chosen !== null && (g.chaptersReached & (1 << r.chosen)) !== 0) startRun(world, g, r.chosen);
  if (r.closed) g.chaptersOpen = false;
}

function records(world: World, g: GameV): void {
  let rows: [number, string, number, number][] = [];
  try {
    rows = g.records === "" ? [] : (JSON.parse(g.records) as [number, string, number, number][]);
  } catch {
    rows = [];
  }
  const entries = rows.map(([rank, name, secs, me]) => ({ rank, name: me ? say(world, "records.you") : name === "" ? say(world, "records.courier", { n: rank }) : name, score: seconds(secs), player: me === 1 }));
  const r = og.ui.leaderboardScreen("Records", {
    title: say(world, "records.title"),
    scopes: [say(world, "records.fastest"), say(world, "records.around")],
    entries,
    loading: g.recordsState === 1,
    error: g.recordsState === 3 ? say(world, "records.error") : "",
    empty: say(world, "records.empty"),
    retryLabel: say(world, "records.retry"),
  });
  if (r.changed) {
    g.recordsScope = r.scope;
    readRecords(world, g);
  }
  if (r.retry) readRecords(world, g);
  if (r.closed) g.recordsOpen = false;
}

function settings(world: World, g: GameV, p: PrefsV): void {
  // Built with its words only while it shows, and for a moment after it closes (its exit):
  // two dozen string lookups a tick are not free (src/state.ts says why that matters here).
  if (!g.settingsOpen && world.tick - g.settingsClosedAt > 30) {
    og.ui.settingsScreen("Settings", { open: false });
    return;
  }
  const r = og.ui.settingsScreen("Settings", {
    open: g.settingsOpen,
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
  if (!g.settingsOpen) return;
  if (r.changed) takeSettings(world, p, r.settings as unknown as Record<string, unknown>);
  if (r.closed) {
    g.settingsOpen = false;
    g.settingsClosedAt = world.tick;
    must(og.platform.saveSettings(world, settingsOf(p)));
    p.saves += 1;
  }
}

function results(world: World, g: GameV): "again" | "title" | "records" | null {
  const won = g.ending === Ending.Rung;
  const sigils = [0, 1, 2].filter((b) => (g.sigils & (1 << b)) !== 0).length;
  const r = og.ui.resultsScreen("Results", {
    open: g.phase === Phase.Over,
    title: say(world, won ? "results.won" : g.ending === Ending.Night ? "results.night" : "results.fallen"),
    score: g.score,
    best: g.bestScore,
    newBest: g.newBest,
    bestLabel: say(world, "results.best"),
    newBestLabel: say(world, "results.newBest"),
    stars: stars(g),
    maxStars: 3,
    stats: [
      { label: say(world, "results.time"), value: seconds(g.time) },
      { label: say(world, "results.sigils"), value: `${sigils}/3` },
      { label: say(world, "results.foes"), value: `${g.kills}` },
      { label: say(world, "results.herbs"), value: `${g.herbsUsed}` },
    ],
    actions: [
      { id: "again", label: say(world, "results.again"), primary: true },
      { id: "records", label: say(world, "results.records") },
      { id: "title", label: say(world, "results.title") },
    ],
  });
  return r.selected === "again" || r.selected === "title" || r.selected === "records" ? r.selected : null;
}

/** The screens, the run's start, clock and end, the hold, the looks, the platform. */
export const Ui = defineSystem(
  "Ui",
  { reads: [Tuning, og.Input, og.Gamepad, OneByOne], writes: [Game, Prefs, Hero, og.Transform, og.CharacterController, og.Audio, og.DayNight, og.RenderSettings, og.PostEffects, og.Stylise, og.MaterialStyle, og.Fog, og.Sky, og.Environment] },
  (_q, time, world) => {
    const g = begin(world);
    try {
      const p = must(world.resource(Prefs));
      const t = must(world.resource(Tuning));
      if (time.tick === 0) {
        readAll(world, g, p);
        applyPrefs(world, p);
      }
      afterRestore(world, g, p);
      answers(world, g, p, (s) => takeSettings(world, p, s));
      if (g.loads > g.welcomed) {
        g.welcomed = g.loads;
        og.ui.toast(world, say(world, "toast.welcome", { place: say(world, `place.${WAYSTONES[g.savePlace]!.key}`) }), { tone: "success" });
      }
      const inPlay = g.phase === Phase.Play;
      const hold = inPlay && (g.paused || g.settingsOpen || g.bagOpen || g.scene !== Scene.None || g.tutorial);
      if (hold !== g.held) {
        must(og.time.setScale(world, hold ? 0 : 1));
        g.held = hold;
      }
      clock(world, g, t, time.scaledDt);
      looks(world, g);
      if (g.musicChunks === 0 && g.music !== 0 && time.tick % 30 === 0) {
        const info = og.load.soundInfo(world, g.music as unknown as SoundHandle);
        if (info.ok && info.value.chunks !== 0) g.musicChunks = info.value.chunks;
      }
      const inp = must(world.resource(og.Input));
      const gp = must(world.resource(og.Gamepad));

      // One wipe for every change of screen, the chapters and the records included.
      const sub = g.chaptersOpen ? "-chapters" : g.recordsOpen ? "-records" : "";
      const screen = (g.phase === Phase.Title ? "title" : g.phase === Phase.Over ? "over" : "play") + (g.phase === Phase.Play ? "" : sub);
      const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "wipe", durationMs: 700 }).shown;
      if (shown.startsWith("title")) {
        og.ui.setNavigation(true);
        if (shown === "title-chapters") chapters(world, g);
        else if (shown === "title-records") records(world, g);
        else {
          const items = [
            ...(g.hasSave ? [{ id: "continue", label: say(world, "title.continue", { place: say(world, `place.${WAYSTONES[g.savePlace]!.key}`), clock: clockText(g.saveClock) }), primary: true }] : []),
            { id: "new", label: say(world, "title.new"), primary: !g.hasSave },
            { id: "chapters", label: say(world, "title.chapters") },
            { id: "records", label: say(world, "title.records") },
            { id: "settings", label: say(world, "title.settings") },
          ];
          const r = og.ui.mainMenu("Title", { title: say(world, "title.name"), subtitle: say(world, "title.tagline"), items, footer: say(world, "title.footer") });
          // The title menu takes no focus by itself, so Enter, Space and the pad's A would do nothing
          // until an arrow or Tab (FINDINGS #14): the focus is put on its first item.
          // (A navigation is applied the tick after it is queued: asked once, not again while it lands.)
          if (og.ui.focus(0) === "" && !g.settingsOpen && world.tick - g.focusAsked > 3) {
            og.ui.navigate(world, "next");
            g.focusAsked = world.tick;
          }
          if (!g.settingsOpen && g.phase === Phase.Title) {
            if (r.selected === "new") startRun(world, g, 0);
            if (r.selected === "continue") continueRun(world, g);
            if (r.selected === "chapters") g.chaptersOpen = true;
            if (r.selected === "records") {
              g.recordsOpen = true;
              readRecords(world, g);
            }
            if (r.selected === "settings") g.settingsOpen = true;
          }
        }
      } else if (shown === "play") {
        const h = g.scene === Scene.None ? hud(world, g) : { bag: false, pause: false };
        const playing = inPlay && g.ending === Ending.None;
        // The cutscenes: the abbess's words at the start, the bell-keeper's at the end.
        if (g.scene !== Scene.None) {
          const lines = g.scene === Scene.Intro ? intro(world) : ending(world);
          og.ui.setNavigation(true);
          const c = og.ui.cutscene("Story", { open: true, lines, index: g.line, letterbox: true, skippable: true, skipLabel: say(world, "story.skip") });
          g.line = c.index;
          if (c.finished || c.skipped) {
            const was = g.scene;
            g.scene = Scene.None;
            g.line = 0;
            if (was === Scene.Intro && !g.tutorialDone) {
              g.tutorial = true;
              g.tutorialStep = 0;
            }
            if (was === Scene.End) g.overTicks = OVER_DELAY;
          }
        } else if (g.tutorial) {
          og.ui.setNavigation(true);
          const r = og.ui.tutorialOverlay("Tutorial", { open: true, steps: tutorialSteps(world), index: g.tutorialStep, skipLabel: say(world, "tut.skip"), nextLabel: say(world, "tut.next"), doneLabel: say(world, "tut.done"), stepLabel: say(world, "tut.step") });
          g.tutorialStep = r.index;
          if (r.finished || r.skipped) {
            g.tutorial = false;
            g.tutorialDone = true;
            writeProgress(world, g);
          }
        } else if (playing && !g.paused && !g.settingsOpen) {
          if (h.pause) g.paused = true;
          if ((h.bag || inp.pressed("Tab") || inp.pressed("KeyI") || gp.pressed("back")) && !g.bagOpen) g.bagOpen = true;
          else if (g.bagOpen && (inp.pressed("Tab") || inp.pressed("KeyI"))) g.bagOpen = false;
          if (!g.bagOpen && inp.pressed("KeyH")) eatHerb(world, g);
        }
        if (g.bagOpen && g.scene === Scene.None) {
          og.ui.setNavigation(true);
          bag(world, g);
        }
        if (g.phase === Phase.Ending && g.scene === Scene.None && g.overTicks >= OVER_DELAY) finish(world, g, t);
        const m = og.ui.pauseMenu("Pause", {
          open: g.paused && !g.settingsOpen,
          title: say(world, "pause.title"),
          items: [
            { id: "resume", label: say(world, "pause.resume"), primary: true },
            { id: "bag", label: say(world, "pause.bag") },
            { id: "settings", label: say(world, "pause.settings") },
            { id: "quit", label: say(world, "pause.quit") },
          ],
        });
        if (m.selected === "resume") g.paused = false;
        if (m.selected === "bag") {
          g.paused = false;
          g.bagOpen = true;
        }
        if (m.selected === "settings") g.settingsOpen = true;
        if (m.selected === "quit") toTitle(world, g);
      } else {
        og.ui.setNavigation(true);
        if (shown === "over-records") records(world, g);
        else {
          const c = results(world, g);
          if (c === "again") startRun(world, g, g.chapter);
          if (c === "title") toTitle(world, g);
          if (c === "records") {
            g.recordsOpen = true;
            readRecords(world, g);
          }
        }
      }
      settings(world, g, p);
      // The accessibility tree, for the playtests: every half second, and on every tick a dialog is open.
      if (!g.settingsOpen && !g.paused && !g.bagOpen && time.tick % 30 !== 0) return;
      const tree = og.ui.a11y();
      const unnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "").length;
      if (unnamed !== g.a11yUnnamed) g.a11yUnnamed = unnamed;
      if (tree.nodes.length !== g.a11yNodes) g.a11yNodes = tree.nodes.length;
    } finally {
      flush(g);
    }
  },
);

export { takeSettings, setLanguage, applyPrefs };
