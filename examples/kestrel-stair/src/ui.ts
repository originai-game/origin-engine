// The Kestrel Stair's screens, on og.ui with the paper style
// (assets/styles/kestrel, brought by the look):
//
// - the title, the chapters, the records, the settings, pause, the bag,
//   the tutorial and the results: the game's own screen documents
//   (assets/ui/*.ogt, ejected from the library's main-menu, saga-map,
//   leaderboard, settings, pause, inventory, tutorial and results with
//   `og ui eject`). The title: Continue with the save slot's place and hour,
//   New journey, Chapters (the four chapters, their stars, the ones not yet
//   reached locked), Records (the fastest ascents and the courier's
//   neighbours, loading, error and empty states honest), Settings
//   (og.ui.settingsTree in its slot), every word from the string tables;
// - the intro and the ending (the game's own letterbox around
//   og.ui.dialogue), the tutorial on a first journey (og.ui.spotlight on
//   each part of the HUD, the card beside it);
// - the HUD: strength and sigils top left, the hour and the bag and pause
//   buttons top right, the prompt for what F would use at the bottom (it
//   shakes when F is refused);
// - the bag (og.ui.grid in its slot: the letter, the sigils, the
//   hillwort; items dragged between slots, hillwort eaten with Use), the
//   pause menu, the results (the stars and the count-up in its slot, under the summit
//   look after a won climb), toasts; a wipe between screens (a fade with
//   reduced motion).
//
// The Ui system also runs the clock, the hold (time scale 0 under a menu),
// the run's start and end, the looks, and og.platform's answers
// (src/save.ts).

import { og, defineSystem } from "@origingame/engine";
import type { Look, SettingEntry, SettingsPage, Sound as SoundHandle, Value, World } from "@origingame/engine";
import { Ending, Game, Hero, HOURS, MAX_HP, Near, Phase, Prefs, Scene, Sound, Tuning } from "./components.ts";
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
const grow = { kind: "grow" } as const;
const px = (n: number) => ({ kind: "fixed", px: n }) as const;

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

/** The bag's slots: item ids by slot, kept as text in Game.bag; six cells a row. */
const BAG_SLOTS = 12;
const BAG_COLUMNS = 6;

/** The game's screen documents (assets/ui/), loaded once at the start. */
export const SCREENS = ["main-menu", "chapters", "leaderboard", "settings", "pause", "bag", "tutorial", "results"] as const;

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

/** The settings as og.platform keeps them (`og.settings`). */
interface Settings {
  masterVolume?: number;
  musicVolume?: number;
  sfxVolume?: number;
  muted?: boolean;
  reducedMotion?: boolean;
  highContrast?: boolean;
  sensitivity?: number;
  invertY?: boolean;
  language?: string;
}

/** The UI's sounds and the player's motion and contrast: og.ui.configure keeps the look's style. */
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

function settingsOf(p: PrefsV): Settings {
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
  g.bagSelected = "";
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
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

function hud(world: World, g: GameV): { bag: boolean; pause: boolean } {
  og.ui.setNavigation(false);
  og.ui.anchor("Status", { anchor: "topLeft", direction: "column", gap: S.sm }, () => {
    capsule("Health", () => {
      og.ui.icon("Heart", "heart", { size: 22, tint: { token: g.hp < 30 ? "danger" : "primary" } });
      og.ui.meter("Strength", g.hp, MAX_HP, { kind: "health", width: px(150) });
    });
    capsule("Sigils", () => {
      for (let i = 0; i < 3; i++) og.ui.icon(`Sigil${i}`, "gem", { size: 20, tint: { token: (g.sigils & (1 << i)) !== 0 ? "primary" : "controlStrong" } });
      og.ui.label("Count", say(world, "hud.sigils", { n: [0, 1, 2].filter((b) => (g.sigils & (1 << b)) !== 0).length }), { font: "mono", nowrap: true });
    });
  });
  let bag = false;
  let pause = false;
  og.ui.anchor("Dusk", { anchor: "topRight", gap: S.sm, cross: "center" }, () => {
    capsule("Clock", () => {
      const left = 1 - (g.hour - HOURS.start) / (HOURS.night - HOURS.start);
      og.ui.icon("Sun", "clock", { size: 20, tint: { token: left < 0.2 ? "danger" : "ink" } });
      og.ui.label("Hour", clockText(g.hour), { font: "mono", nowrap: true, width: px(48) });
      og.ui.meter("Daylight", Math.max(0, left), 1, { kind: "primary", width: px(90) });
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
        og.ui.icon("Key", g.nearKind === Near.Gate && (g.sigils & 7) !== 7 ? "lock" : "flag", { size: 18, tint: { token: "primary" } });
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

/**
 * The bag as the game shipped it: the letter, the sigils carried, the hillwort in twelve slots six a row (og.ui.grid,
 * the slots sharing the width beside a 200 px detail); a press selects, a drag moves an item to another slot (the
 * slots kept as text in Game.bag), Use eats a hillwort.
 */
function bag(world: World, g: GameV): void {
  let slots: string[];
  try {
    slots = JSON.parse(g.bag) as string[];
  } catch {
    slots = [];
  }
  while (slots.length < BAG_SLOTS) slots.push("");
  const carried = (id: string) => id === "letter" || (id.startsWith("sigil") && (g.sigils & (1 << Number(id.slice(5)))) !== 0) || (id === "herb" && g.herbs > 0);
  const icon = (id: string) => (id === "letter" ? "map" : id === "herb" ? "leaf" : "gem");
  const count = (id: string) => (id === "herb" ? g.herbs : 0);
  if (g.bagSelected !== "" && !(slots.includes(g.bagSelected) && carried(g.bagSelected))) g.bagSelected = "";
  // assets/ui/bag.ogt: the dialog panel and its title; the slots and the detail in its Body slot.
  const asked = screen(world, "bag", true);
  let used: string | null = null;
  if (og.ui.openSlot("bag/Body")) {
    const phone = og.ui.frameViewport().breakpoint.startsWith("phone");
    const cell = phone ? 64 : 72;
    og.ui[phone ? "column" : "row"]("Body", { width: grow, gap: phone ? S.md : S.lg }, () => {
      const sel = g.bagSelected === "" ? -1 : slots.indexOf(g.bagSelected);
      const r = og.ui.grid("Slots", { count: BAG_SLOTS, cellWidth: cell, cellHeight: cell, columns: BAG_COLUMNS, height: px(phone ? 300 : 340), width: grow, selected: sel >= 0 ? sel : null }, (i) => {
        const id = slots[i] ?? "";
        if (!carried(id)) return;
        og.ui.icon("Picture", icon(id), { size: cell * 0.5 });
        if (count(id) > 1) og.ui.label("Count", `${count(id)}`, { step: "xs", font: "mono", color: { token: "inkMuted" }, nowrap: true });
      });
      if (r.clicked !== null) g.bagSelected = carried(slots[r.clicked] ?? "") ? slots[r.clicked]! : "";
      if (r.dropFrom !== null && r.dropTo !== null && carried(slots[r.dropFrom] ?? "") && r.dropTo !== r.dropFrom && r.dropTo < BAG_SLOTS) {
        const moving = slots[r.dropFrom]!;
        slots[r.dropFrom] = slots[r.dropTo] ?? "";
        slots[r.dropTo] = moving;
        g.bag = JSON.stringify(slots);
        g.bagMoves += 1;
        g.bagSelected = moving;
      }
      const it = g.bagSelected;
      og.ui.column("Detail", { width: phone ? grow : px(200), gap: S.sm }, () => {
        if (it === "") {
          og.ui.label("Hint", say(world, "bag.hint"), { step: "meta", color: { token: "inkMuted" }, width: grow });
          return;
        }
        og.ui.label("Name", say(world, `item.${it}`), { step: "lg", width: grow });
        og.ui.label("Description", say(world, `item.${it}.about`), { step: "meta", color: { token: "inkMuted" }, width: grow });
        if (count(it) > 1) og.ui.keyValue("Count", say(world, "bag.count"), `${count(it)}`, true);
        if (og.ui.button("Use", say(world, "bag.use"), { variant: "primary", width: grow }).clicked) used = it;
      });
    });
    og.ui.close();
  }
  if (used === "herb") eatHerb(world, g);
  if (asked.includes("close")) g.bagOpen = false;
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

/**
 * A cutscene in the film's frame, as the game shipped it: a black bar a tenth of the screen top and bottom (each an
 * anchor, so it keeps the style's own frame, its panel's edge, round the black), the line in og.ui.dialogue straddling
 * the bottom bar 4 % of the screen up, typed out (a press or Enter shows it whole, the next one goes on), and Skip at
 * the top right under the bar, as Back is.
 */
function letterboxTalk(world: World, lines: { speaker: string; text: string; portrait: string }[], index: number): { index: number; finished: boolean; skipped: boolean } {
  const out = { index, finished: false, skipped: false };
  og.ui.setNavigation(true);
  const vp = og.ui.frameViewport();
  const black = { color: { r: 0, g: 0, b: 0 } };
  for (const [name, anchor] of [["Top", "top"], ["Bottom", "bottom"]] as const) {
    og.ui.anchor(`Story.${name}`, { anchor, margin: 0, width: grow, height: { kind: "percent", fraction: 0.1 }, background: black }, () => {});
  }
  const line = lines[Math.min(index, lines.length - 1)]!;
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  og.ui.anchor("Story.box", { anchor: "bottom", width: phone ? grow : px(Math.min(760, vp.width - 64)), offsetY: -vp.height * 0.04 }, () => {
    const r = og.ui.dialogue("Dialogue", { speaker: line.speaker, text: line.text, portraitIcon: line.portrait });
    if (r.advanced) {
      if (index + 1 >= lines.length) out.finished = true;
      else out.index = index + 1;
    }
  });
  // The line takes the focus as it appears, but not while the wipe from the title still moves (the screens are inert
  // then): it is given it once nothing holds it, so Enter, Space and the pad's A walk the lines.
  if (og.ui.focus(0) === "") og.ui.setFocus("Story.box/Dialogue");
  og.ui.anchor("Story.skip", { anchor: "topRight", offsetY: vp.height * 0.1 }, () => {
    if (og.ui.button("Skip", say(world, "story.skip"), { variant: "quiet", size: "small" }).clicked) out.skipped = true;
  });
  if (og.ui.back()) out.skipped = true;
  return out;
}

function tutorialSteps(world: World) {
  return [
    { target: "Health", title: say(world, "tut.health.title"), text: say(world, "tut.health.text") },
    { target: "Sigils", title: say(world, "tut.sigils.title"), text: say(world, "tut.sigils.text") },
    { target: "Clock", title: say(world, "tut.clock.title"), text: say(world, "tut.clock.text") },
    { target: "Bag", title: say(world, "tut.bag.title"), text: say(world, "tut.bag.text") },
  ];
}

/** The card's room from the screen's edges and from the part it points at. */
const GUTTER = 16;

/**
 * The tutorial (assets/ui/tutorial.ogt): og.ui.spotlight on the step's part of the HUD under it, the card under
 * the part where there is room, else over it; Next walks the steps, Done (the last step's Next) and Skip
 * (Back too) end it.
 */
function tutorial(world: World, g: GameV): { finished: boolean } {
  const steps = tutorialSteps(world);
  const index = Math.min(g.tutorialStep, steps.length - 1);
  const step = steps[index]!;
  const vp = og.ui.frameViewport();
  const safe = vp.safeArea;
  const hole = og.ui.bounds(step.target);
  const width = Math.min(320, vp.width - 2 * GUTTER);
  const height = og.ui.bounds("Tutorial/Card")?.height ?? 160;
  let x = vp.width / 2 - width / 2;
  let y = vp.height / 2 - height / 2;
  if (hole) {
    x = Math.min(Math.max(hole.x, safe.left + GUTTER), vp.width - safe.right - GUTTER - width);
    const below = hole.y + hole.height + S.lg;
    y = below + height <= vp.height - safe.bottom - GUTTER ? below : Math.max(safe.top + GUTTER, hole.y - S.lg - height);
  }
  const last = index + 1 >= steps.length;
  // The scrim with the part left clear, under the document's layer.
  og.ui.spotlight("Spotlight", hole, {});
  const asked = screen(world, "tutorial", true, {
    count: say(world, "tut.step", { step: index + 1, count: steps.length }),
    title: step.title,
    text: step.text,
    nextLabel: say(world, last ? "tut.done" : "tut.next"),
    x,
    y,
  });
  if (asked.includes("skip")) return { finished: true };
  if (asked.includes("next")) {
    if (last) return { finished: true };
    g.tutorialStep = index + 1;
  }
  return { finished: false };
}

function chapters(world: World, g: GameV): void {
  // assets/ui/chapters.ogt: the dialog panel and its title; in its Body slot the stars earned of all, the four
  // chapters as two wide cards a row (og.ui.grid), each named whole (FINDINGS #15) with its stars, one not yet reached
  // a lock and refused, and Back (Escape, the pad's B and a tap on the scrim close it too), as the game shipped them.
  const phone = og.ui.frameViewport().breakpoint.startsWith("phone");
  const levels = CHAPTERS.map((c, i) => ({ name: say(world, `chapter.${c.key}`), stars: (g.chapterStars >> (i * 2)) & 3, locked: (g.chaptersReached & (1 << i)) === 0 }));
  const earned = levels.reduce((n, l) => n + l.stars, 0);
  const asked = screen(world, "chapters", true);
  let back = asked.includes("back");
  if (og.ui.openSlot("chapters/Body")) {
    og.ui.row("Total", { gap: S.xs, cross: "center" }, () => {
      og.ui.icon("Star", "star", { size: 18, tint: { token: "gold" }, label: say(world, "chapters.stars", { stars: earned, max: CHAPTERS.length * 3 }) });
      og.ui.label("Count", `${earned} / ${CHAPTERS.length * 3}`, { font: "mono", color: { token: "inkMuted" }, nowrap: true });
    });
    const cell = phone ? 84 : 104;
    const r = og.ui.grid("Levels", { count: levels.length, cellWidth: phone ? 136 : 168, cellHeight: cell, columns: 2, height: { kind: "fit", min: 0, max: phone ? 420 : 400 }, width: grow }, (i) => {
      const l = levels[i]!;
      if (l.locked) {
        og.ui.icon("Lock", "lock", { size: 28, tint: { token: "inkFaint" }, label: say(world, "chapters.locked", { name: l.name }) });
        return;
      }
      og.ui.label("Name", l.name, { step: phone ? "body" : "lg", font: "display", align: "center", width: grow, maxLines: 2, ellipsis: true });
      og.ui.row("Stars", { gap: 2 }, () => {
        for (let k = 0; k < 3; k++) og.ui.icon(og.ui.idi("Star", k), "star", { size: 14, tint: { token: k < l.stars ? "gold" : "controlStrong" }, label: k === 0 ? say(world, "chapters.stars", { stars: l.stars, max: 3 }) : "" });
      });
    });
    og.ui.row("Footer", { width: grow, align: "end" }, () => {
      if (og.ui.button("Back", say(world, "chapters.back")).clicked) back = true;
    });
    og.ui.close();
    if (r.clicked !== null && !levels[r.clicked]!.locked) {
      startRun(world, g, r.clicked);
      return;
    }
  }
  if (back) g.chaptersOpen = false;
}

/** The records' medals: ranks 1 to 3 on a metal disc (gold, silver, bronze: its light, its body and its shade). */
const MEDALS = [
  [{ r: 1, g: 0.902, b: 0.502 }, { r: 1, g: 0.8, b: 0 }, { r: 0.722, g: 0.525, b: 0.043 }],
  [{ r: 0.957, g: 0.965, b: 0.973 }, { r: 0.722, g: 0.753, b: 0.8 }, { r: 0.478, g: 0.51, b: 0.557 }],
  [{ r: 0.953, g: 0.776, b: 0.627 }, { r: 0.788, g: 0.514, b: 0.302 }, { r: 0.541, g: 0.294, b: 0.133 }],
] as const;

function records(world: World, g: GameV): void {
  let rows: [number, string, number, number][] = [];
  try {
    rows = g.records === "" ? [] : (JSON.parse(g.records) as [number, string, number, number][]);
  } catch {
    rows = [];
  }
  // assets/ui/leaderboard.ogt: the dialog panel and its title; in its Body slot the two scopes as tabs, then the
  // fastest ascents or the courier's neighbours (the courier's row lit, ranks 1 to 3 on medals), loading (skeleton
  // rows, never invented times), error (Try again) or empty, as the game shipped them. Back (Escape, B) closes it.
  const phone = og.ui.frameViewport().breakpoint.startsWith("phone");
  const sizes = og.ui.frameTheme().sizes;
  const asked = screen(world, "leaderboard", true);
  if (og.ui.openSlot("leaderboard/Body")) {
    const tabs = og.ui.tabs("Scopes", [say(world, "records.fastest"), say(world, "records.around")], g.recordsScope, { fill: true, width: grow });
    if (tabs.response.changed) {
      g.recordsScope = tabs.selected;
      readRecords(world, g);
    }
    const height = px(phone ? 380 : 360);
    if (g.recordsState === 1) {
      og.ui.column("Loading", { width: grow, gap: S.md, height }, () => {
        for (let i = 0; i < 6; i++) og.ui.skeleton(og.ui.idi("Row", i), { height: sizes.controlSm });
      });
    } else if (g.recordsState === 3) {
      og.ui.column("Error", { width: grow, gap: S.md, height, align: "center", cross: "center" }, () => {
        og.ui.label("Message", say(world, "records.error"), { color: { token: "danger" }, align: "center", width: grow });
        if (og.ui.button("Retry", say(world, "records.retry")).clicked) readRecords(world, g);
      });
    } else if (rows.length === 0) {
      og.ui.column("Empty", { width: grow, height, align: "center", cross: "center" }, () => {
        og.ui.label("Message", say(world, "records.empty"), { color: { token: "inkMuted" }, align: "center", width: grow });
      });
    } else {
      const player = rows.findIndex((r) => r[3] === 1);
      og.ui.list("Rows", { count: rows.length, rowHeight: sizes.row, height, width: grow, selected: player >= 0 ? player : null }, (i) => {
        const [rank, name, secs, me] = rows[i]!;
        og.ui.row("Rank", { width: px(40), cross: "center" }, () => {
          if (rank >= 1 && rank <= 3) {
            const [hi, mid, shade] = MEDALS[rank - 1]!;
            og.ui.row("Medal", { width: px(26), height: px(26), align: "center", cross: "center" }, () => {
              og.ui.setDraw({
                shape: { kind: "ellipse" },
                fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: { color: hi } }, { offset: 0.5, color: { color: mid } }, { offset: 1, color: { color: shade } }] }],
                effects: [{ kind: "innerShadow", offset: { x: 0, y: 1 }, blur: 0, spread: 0, color: { color: hi, alpha: 0.9 } }],
              } as Parameters<typeof og.ui.setDraw>[0]);
              og.ui.richText("Numeral", `[b]${rank}[/b]`, { font: "mono", step: "meta", color: { color: { r: 0.082, g: 0.082, b: 0.102 } }, nowrap: true });
            });
          } else {
            og.ui.label("Numeral", `${rank}`, { font: "mono", color: { token: "inkMuted" }, nowrap: true });
          }
        });
        og.ui.label("Name", me ? say(world, "records.you") : name === "" ? say(world, "records.courier", { n: rank }) : name, { width: grow, nowrap: true });
        og.ui.label("Score", seconds(secs), { font: "mono", nowrap: true });
      });
    }
    og.ui.close();
  }
  if (asked.includes("back")) g.recordsOpen = false;
}

/** The settings tree's pages: what Prefs keeps, every word from the tables. */
function pages(world: World, p: PrefsV): SettingsPage[] {
  const volume = (key: string, label: string, value: number): SettingEntry => ({ key, kind: "slider", label: say(world, label), value, min: 0, max: 1, step: 0.05, unit: "%" });
  const toggle = (key: string, label: string, on: boolean): SettingEntry => ({ key, kind: "toggle", label: say(world, label), on });
  return [
    { key: "audio", label: say(world, "settings.audio"), entries: [volume("master", "settings.master", p.master), volume("music", "settings.music", p.music), volume("effects", "settings.effects", p.effects), toggle("muted", "settings.mute", p.muted)] },
    { key: "graphics", label: say(world, "settings.graphics"), entries: [toggle("reducedMotion", "settings.motion", p.reducedMotion), toggle("highContrast", "settings.contrast", p.highContrast)] },
    { key: "controls", label: say(world, "settings.controls"), entries: [{ key: "sensitivity", kind: "slider", label: say(world, "settings.sensitivity"), value: p.sensitivity, min: 0.1, max: 3, step: 0.1 }, toggle("invertY", "settings.invert", p.invertY)] },
    { key: "language", label: say(world, "settings.language"), entries: [{ key: "language", kind: "choice", label: say(world, "settings.language"), options: LANGUAGES.map((l) => say(world, l.key)), index: p.language }] },
  ];
}

/** The defaults of a page's entries (Defaults). */
const DEFAULTS: Record<string, Settings> = {
  audio: { masterVolume: 1, musicVolume: 0.8, sfxVolume: 1, muted: false },
  graphics: { reducedMotion: false, highContrast: false },
  controls: { sensitivity: 1, invertY: false },
  language: { language: "en" },
};

function settings(world: World, g: GameV, p: PrefsV): void {
  // assets/ui/settings.ogt, its Tree slot og.ui.settingsTree; every change is applied at once, saved on Done.
  const asked = screen(world, "settings", g.settingsOpen);
  if (!g.settingsOpen) return;
  if (og.ui.openSlot("settings/Tree")) {
    const r = og.ui.settingsTree(world, "Tree", {
      title: "",
      pages: pages(world, p),
      page: g.settingsPage,
      applyLabel: say(world, "settings.apply"),
      revertLabel: say(world, "settings.revert"),
      defaultsLabel: say(world, "settings.defaults"),
      changedLabel: say(world, "settings.changed"),
    }, () => {});
    og.ui.close();
    if (r.page !== null) g.settingsPage = r.page;
    const c = r.change;
    if (c !== null) {
      const s: Settings = {};
      if (c.key === "master") s.masterVolume = c.value;
      else if (c.key === "music") s.musicVolume = c.value;
      else if (c.key === "effects") s.sfxVolume = c.value;
      else if (c.key === "muted") s.muted = c.on;
      else if (c.key === "reducedMotion") s.reducedMotion = c.on;
      else if (c.key === "highContrast") s.highContrast = c.on;
      else if (c.key === "sensitivity") s.sensitivity = c.value;
      else if (c.key === "invertY") s.invertY = c.on;
      else if (c.key === "language") s.language = LANGUAGES[c.index]?.code;
      takeSettings(world, p, s as Record<string, unknown>);
    }
    if (r.defaults) takeSettings(world, p, (DEFAULTS[g.settingsPage || "audio"] ?? {}) as Record<string, unknown>);
  }
  if (asked.includes("done")) {
    g.settingsOpen = false;
    g.settingsClosedAt = world.tick;
    must(og.platform.saveSettings(world, settingsOf(p)));
    p.saves += 1;
  }
}

/** A count as the results show it: 12,840. */
function grouped(n: number): string {
  return String(Math.floor(n)).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

/**
 * The results as the game shipped them (assets/ui/results.ogt: the dialog panel and how the climb ended; its Body slot
 * here): the stars landing one by one 0.3 s apart from the opening (an earned one gold, with a burst), the score
 * counting up over 0.9 s, the best and the New record badge popping in as it lands, the time, the sigils, the foes and
 * the hillwort eaten, then Again, Records and Title.
 */
function results(world: World, g: GameV): "again" | "title" | "records" | null {
  const won = g.ending === Ending.Rung;
  const sigils = [0, 1, 2].filter((b) => (g.sigils & (1 << b)) !== 0).length;
  const phone = og.ui.frameViewport().breakpoint.startsWith("phone");
  // When they opened, in og.ui's seconds: the count-up and the stars run from there (dropped when the results go).
  let since = og.ui.memory("resultsSince", -1);
  if (since < 0) since = og.ui.seconds();
  og.ui.remember("resultsSince", since);
  const age = og.ui.seconds() - since;
  screen(world, "results", true, { title: say(world, won ? "results.won" : g.ending === Ending.Night ? "results.night" : "results.fallen") });
  let chosen: "again" | "title" | "records" | null = null;
  if (og.ui.openSlot("results/Body")) {
    const earned = stars(g);
    og.ui.row("StarRow", { width: grow, align: "center" }, () => {
      og.ui.row("Stars", { gap: S.sm, cross: "center" }, () => {
        for (let k = 0; k < 3; k++) {
          const shown = age >= 0.3 * (k + 1);
          const on = k < earned;
          og.ui.transition(og.ui.idi("Star", k), shown, { kind: "scale", instant: false }, () => {
            og.ui.icon("Mark", "star", { size: 44, tint: { token: on ? "gold" : "controlStrong" } });
            if (on) og.ui.burst({ delayMs: 120 });
          });
          if (!shown) og.ui.spacer(og.ui.idi("Gap", k), px(44));
        }
      });
    });
    og.ui.column("ScoreBlock", { width: grow, cross: "center", gap: 2 }, () => {
      og.ui.counter("Score", g.score, { from: 0, durationMs: 900, step: "displayLg", font: "display", align: "center", width: grow });
      og.ui.row("BestRow", { gap: S.sm, cross: "center" }, () => {
        og.ui.label("Best", say(world, "results.best", { best: grouped(Math.max(g.bestScore, g.newBest ? g.score : 0)) }), { step: "meta", color: { token: "inkMuted" }, nowrap: true });
        if (g.newBest) og.ui.transition("Reveal", age >= 0.9, { kind: "pop" }, () => og.ui.badge("NewBest", say(world, "results.newBest"), "primary"));
      });
    });
    const rows = [
      [say(world, "results.time"), seconds(g.time)],
      [say(world, "results.sigils"), `${sigils}/3`],
      [say(world, "results.foes"), `${g.kills}`],
      [say(world, "results.herbs"), `${g.herbsUsed}`],
    ] as const;
    og.ui.column("Stats", { width: grow, gap: 0 }, () => {
      rows.forEach(([label, value], i) => og.ui.keyValue(og.ui.idi("Stat", i), label, value, true));
    });
    og.ui.row("Actions", { width: grow, gap: S.sm, wrap: phone }, () => {
      if (og.ui.button("again", say(world, "results.again"), { variant: "primary", width: grow }).clicked) chosen = "again";
      if (og.ui.button("records", say(world, "results.records"), { width: grow }).clicked) chosen = "records";
      if (og.ui.button("title", say(world, "results.title"), { width: grow }).clicked) chosen = "title";
    });
    og.ui.close();
  }
  return chosen;
}

/** The screens, the run's start, clock and end, the hold, the looks, the platform. */
export const Ui = defineSystem(
  "Ui",
  { reads: [Tuning, og.Input, og.Gamepad], writes: [Game, Prefs, Hero, og.Transform, og.CharacterController, og.Audio, og.DayNight, og.RenderSettings, og.PostEffects, og.Stylise, og.MaterialStyle, og.Fog, og.Sky, og.Environment] },
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
      const want = (g.phase === Phase.Title ? "title" : g.phase === Phase.Over ? "over" : "play") + (g.phase === Phase.Play ? "" : sub);
      const shown = og.ui.screenTransition("Screens", want, { kind: p.reducedMotion ? "fade" : "wipe", durationMs: 700 }).shown;
      if (shown !== "title") g.leftTitle = true;
      // The screen documents of the screens not shown leave the layer stack first: popped after the new screen is
      // built, a layer would hand the focus back over what that screen just focused (the cutscene's line).
      if (shown !== "title") screen(world, "main-menu", false);
      if (shown !== "title-chapters") screen(world, "chapters", false);
      if (shown !== "title-records" && shown !== "over-records") screen(world, "leaderboard", false);
      if (shown !== "over") screen(world, "results", false);
      if (shown !== "play") screen(world, "pause", false);
      let tutorialUp = false;
      let bagUp = false;
      if (shown.startsWith("title")) {
        og.ui.setNavigation(true);
        if (shown === "title-chapters") chapters(world, g);
        else if (shown === "title-records") records(world, g);
        else {
          // assets/ui/main-menu.ogt: Continue (the place and the hour; primary and focused, FINDINGS #14), New journey,
          // Chapters, Records, Settings. It has no entrance: the web draws the title the game opens on behind the play
          // gate, where an entrance would stand part-way until the tap (og-platform-sdk §3).
          const asked = screen(world, "main-menu", true, {
            hasSave: g.hasSave,
            continueLine: g.hasSave ? say(world, "title.continue", { place: say(world, `place.${WAYSTONES[g.savePlace]!.key}`), clock: clockText(g.saveClock) }) : "",
          });
          // The save is read after the title first shows (New journey focused then): Continue takes the focus once it
          // appears, as the primary action.
          const focused = og.ui.memory("continueFocused", 0);
          if (g.hasSave && focused === 0) og.ui.setFocus("main-menu/Title/Card/Items/continue");
          og.ui.remember("continueFocused", g.hasSave ? 1 : 0);
          if (!g.settingsOpen && g.phase === Phase.Title) {
            if (asked.includes("new")) startRun(world, g, 0);
            else if (asked.includes("continue")) continueRun(world, g);
            else if (asked.includes("chapters")) g.chaptersOpen = true;
            else if (asked.includes("records")) {
              g.recordsOpen = true;
              readRecords(world, g);
            } else if (asked.includes("settings")) g.settingsOpen = true;
          }
        }
      } else if (shown === "play") {
        const h = g.scene === Scene.None ? hud(world, g) : { bag: false, pause: false };
        const playing = inPlay && g.ending === Ending.None;
        // The cutscenes: the abbess's words at the start, the bell-keeper's at the end.
        if (g.scene !== Scene.None) {
          const lines = g.scene === Scene.Intro ? intro(world) : ending(world);
          const c = letterboxTalk(world, lines, g.line);
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
          tutorialUp = true;
          if (tutorial(world, g).finished) {
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
          bagUp = true;
          bag(world, g);
        }
        if (g.phase === Phase.Ending && g.scene === Scene.None && g.overTicks >= OVER_DELAY) finish(world, g, t);
        // assets/ui/pause.ogt: Resume (and Back), Bag, Settings, Rest and quit.
        for (const e of screen(world, "pause", g.paused && !g.settingsOpen)) {
          if (e === "resume") g.paused = false;
          if (e === "bag") {
            g.paused = false;
            g.bagOpen = true;
          }
          if (e === "settings") g.settingsOpen = true;
          if (e === "quit") toTitle(world, g);
        }
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
      // The bag and the tutorial leave the layer stack once they are done.
      if (!bagUp) screen(world, "bag", false);
      if (!tutorialUp) screen(world, "tutorial", false);
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
