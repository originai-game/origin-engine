// Brasswick's screens on og.ui, dressed by the brasswick style (the fantasy
// pack recoloured; the look applies it): the title over the first level
// (Continue when the vault slot answered, New game, Levels, Settings), the
// level select (stars, best gears and time, locks), the HUD (gears and the
// level, touch controls on a phone), the pause menu, the settings (tabs
// over their rows, every word from the string tables, English and Spanish)
// and the results (the score counting up, the stars, the best). The title,
// the level select, pause, the settings and the results are the game's own
// screen documents (assets/ui/*.ogt, ejected with `og ui eject` from the
// library's main-menu, saga-map, pause, settings and results), laid out as
// the game shipped them on the fixed templates; what a document cannot say
// (the level cards' grid, the settings' tabs and rows, the stars landing)
// is built here in its slots.
// The screens change through an iris (a fade with reduced motion).
//
// The Ui system also starts levels, holds the world (time scale 0) while
// paused or in the settings, counts the level's clock, and finishes a level
// when the exit's cheer is over (the score, the stars, the progress write).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, GEARS, LEVELS, Phase, Prefs, Tuning, Zone } from "./components.ts";
import { say } from "./feel.ts";
import { intent } from "./hero.ts";
import { must } from "./level.ts";
import { LANGUAGES, settingsOf, takeSettings, type Settings } from "./prefs.ts";
import { askContinue } from "./save.ts";

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

const LEVEL_KEYS = ["level.1", "level.2", "level.3"] as const;

/** The game's screen documents (assets/ui/), loaded once at the start. */
export const SCREENS = ["main-menu", "levels", "pause", "settings", "results"] as const;

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

/** m:ss.t */
export function clock(seconds: number): string {
  const s = Math.max(0, seconds);
  const m = Math.floor(s / 60);
  const r = s - m * 60;
  return `${m}:${r < 10 ? "0" : ""}${r.toFixed(1)}`;
}

export interface Best {
  gears: number;
  stars: number;
  time: number;
  score: number;
}

export function bestOf(g: GameV, n: number): Best {
  return n === 0
    ? { gears: g.best0, stars: g.stars0, time: g.time0, score: g.top0 }
    : n === 1
      ? { gears: g.best1, stars: g.stars1, time: g.time1, score: g.top1 }
      : { gears: g.best2, stars: g.stars2, time: g.time2, score: g.top2 };
}

function setBest(g: GameV, n: number, b: Best): void {
  if (n === 0) {
    g.best0 = b.gears;
    g.stars0 = b.stars;
    g.time0 = b.time;
    g.top0 = b.score;
  } else if (n === 1) {
    g.best1 = b.gears;
    g.stars1 = b.stars;
    g.time1 = b.time;
    g.top1 = b.score;
  } else {
    g.best2 = b.gears;
    g.stars2 = b.stars;
    g.time2 = b.time;
    g.top2 = b.score;
  }
}

/** Starts level n (0-based) from its spawn: the level's things put back, Wick warped there, its light. */
export function startLevel(world: World, g: GameV, n: number): void {
  g.level = n;
  g.phase = Phase.Play;
  g.levelsOpen = g.paused = g.settingsOpen = false;
  g.time = 0;
  g.gears = g.gearMask = g.deaths = g.stomps = 0;
  g.exiting = g.hurt = g.safe = 0;
  g.hurtAsk = g.bounce = false;
  g.checkpoint = -1;
  g.reset = 2;
  g.resetLevel = n;
  placeAtSpawn(world, g, n);
}

/** The title: the first level as it starts, Wick standing at its spawn. */
export function toTitle(world: World, g: GameV): void {
  startLevel(world, g, 0);
  g.phase = Phase.Title;
}

function placeAtSpawn(world: World, g: GameV, n: number): void {
  const spawn = world.find(`l${n + 1}-spawn`);
  if (spawn === null) throw new Error(`level ${n + 1} has no spawn`);
  const p = must(world.get(spawn, og.Transform)).position;
  g.spawnX = g.checkX = g.warpX = p.x;
  g.spawnY = g.checkY = g.warpY = p.y;
  g.warp = true;
  g.heroX = p.x;
  g.heroY = p.y;
  const zone = world.find(`l${n + 1}-zone`);
  if (zone !== null) {
    const z = must(world.get(zone, Zone));
    const lighting = must(world.resource(og.Lighting2D));
    lighting.ambient = { r: z.ambient, g: z.ambientG, b: z.ambientB };
  }
}

/** The level is over (the cheer ran out): the score, the stars, the best, and the progress written. */
export function finishLevel(world: World, g: GameV): void {
  const n = g.level;
  const zone = world.find(`l${n + 1}-zone`);
  const par = zone === null ? 60 : must(world.get(zone, Zone)).par;
  const all = g.gears === GEARS[n];
  const quick = g.time <= par;
  g.stars = 1 + (all ? 1 : 0) + (quick ? 1 : 0);
  g.score = g.gears * 100 + Math.max(0, Math.round((par - g.time) * 10)) + (g.deaths === 0 ? 200 : 0);
  const best = bestOf(g, n);
  g.newBest = g.score > best.score;
  setBest(g, n, {
    gears: Math.max(best.gears, g.gears),
    stars: Math.max(best.stars, g.stars),
    time: best.time === 0 ? g.time : Math.min(best.time, g.time),
    score: Math.max(best.score, g.score),
  });
  g.unlocked = Math.max(g.unlocked, Math.min(LEVELS, n + 2));
  g.phase = Phase.Results;
  g.exiting = 0;
  g.writeNow = true;
}

function title(world: World, g: GameV): void {
  // assets/ui/main-menu.ogt: Continue (with the saved level), New game, Levels, Settings.
  const asked = screen(world, "main-menu", true, {
    canContinue: g.hasSlot,
    continueLine: g.hasSlot ? say(world, "title.continueAt", { level: g.slotLevel + 1 }) : say(world, "title.continue"),
  });
  if (g.levelsOpen || g.settingsOpen) return;
  if (asked.includes("continue")) askContinue(world, g);
  else if (asked.includes("new")) startLevel(world, g, 0);
  else if (asked.includes("levels")) g.levelsOpen = true;
  else if (asked.includes("settings")) g.settingsOpen = true;
}

function levels(world: World, g: GameV): void {
  // assets/ui/levels.ogt: the title, the stars earned of all and Back (a tap on the scrim, Escape and the pad's B are
  // Back too: FINDINGS.md #10); its Levels slot the cards as they shipped, a grid of three: an open card its number
  // and name in display type and its stars (the first star named with them for a screen reader), a locked one a lock
  // named by its level, refused.
  const earned = LEVEL_KEYS.reduce((sum, _key, n) => sum + Math.min(3, bestOf(g, n).stars), 0);
  const asked = screen(world, "levels", g.levelsOpen, { earned: `${earned} / ${LEVELS * 3}`, earnedLabel: say(world, "levels.stars", { stars: earned, max: LEVELS * 3 }) });
  if (!g.levelsOpen) return;
  let picked = -1;
  if (og.ui.openSlot("levels/Levels")) {
    const r = og.ui.grid("Levels", { count: LEVELS, cellWidth: 168, cellHeight: 104, columns: 3, height: { kind: "fit", min: 0, max: 400 }, width: { kind: "grow" } }, (n) => {
      const name = say(world, "levels.card", { n: n + 1, name: say(world, LEVEL_KEYS[n]!) });
      if (n >= g.unlocked) {
        og.ui.icon("Lock", "lock", { size: 28, tint: { token: "inkFaint" }, label: say(world, "levels.locked", { name }) });
        return;
      }
      og.ui.label("Name", name, { step: "lg", font: "display", align: "center", width: { kind: "grow" }, maxLines: 2, ellipsis: true });
      const stars = Math.min(3, bestOf(g, n).stars);
      og.ui.row("Stars", { gap: 2 }, () => {
        for (let k = 0; k < 3; k++) og.ui.icon(og.ui.idi("Star", k), "star", { size: 14, tint: { token: k < stars ? "gold" : "controlStrong" }, label: k === 0 ? say(world, "levels.stars", { stars, max: 3 }) : "" });
      });
    });
    og.ui.close();
    if (r.clicked !== null && r.clicked < g.unlocked) picked = r.clicked;
  }
  if (picked >= 0) startLevel(world, g, picked);
  else if (asked.includes("back")) g.levelsOpen = false;
}

const S = { xs: 4, sm: 8, md: 12 } as const;

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, build);
}

/**
 * The HUD, from widgets: a puzzle game without lives or a score builds its own (og-taste §2: capsules
 * with an icon and a number, in the corners).
 */
function hud(world: World, g: GameV): void {
  // All in the top-right corner: on the published page the shell's Back handle sits over the top-left
  // one, outside og.ui's safe area (FINDINGS.md #15).
  let paused = false;
  og.ui.anchor("Hud", { anchor: "topRight", direction: "row", gap: S.sm, cross: "center" }, () => {
    capsule("Count", () => {
      og.ui.icon("Gear", "gear", { size: 20, tint: { token: "primary" } });
      og.ui.label("Gears", say(world, "hud.gears", { n: g.gears, of: GEARS[g.level] }), { step: "lg", nowrap: true });
    });
    capsule("Name", () => {
      og.ui.label("Text", say(world, "levels.card", { n: g.level + 1, name: say(world, LEVEL_KEYS[g.level]!) }), { nowrap: true });
    });
    if (og.ui.actionButton("Pause", say(world, "hud.pause"), { icon: "pause", size: 44 }).clicked) paused = true;
  });
  // Touch controls on a phone or tablet: a d-pad and Jump (og-ui §5).
  g.touchX = 0;
  g.touchJump = false;
  if (og.ui.frameViewport().coarsePointer) {
    og.ui.anchor("Pad", { anchor: "bottomLeft" }, () => {
      const d = og.ui.dpad("Dpad", { size: 132 });
      g.touchX = (d.right ? 1 : 0) - (d.left ? 1 : 0);
    });
    og.ui.anchor("Act", { anchor: "bottomRight" }, () => {
      g.touchJump = og.ui.actionButton("Jump", say(world, "hud.jump"), { size: 72, primary: true }).pressed;
    });
  }
  if (paused && !g.paused) g.paused = true;
}

function pause(world: World, g: GameV): void {
  // assets/ui/pause.ogt: Resume (and Back), Back to the lamp, Restart the level, Settings, Quit to the title.
  for (const e of screen(world, "pause", g.paused && !g.settingsOpen)) {
    switch (e) {
      case "resume":
        g.paused = false;
        break;
      case "retry":
        g.paused = false;
        g.hurt = 1;
        g.deaths += 1;
        break;
      case "restart":
        startLevel(world, g, g.level);
        break;
      case "settings":
        g.settingsOpen = true;
        break;
      case "quit":
        toTitle(world, g);
        break;
    }
  }
}

/** The settings' sections, in their tabs' order (the string key of each one's name). */
const SECTIONS = ["settings.audio", "settings.graphics", "settings.controls", "settings.language"] as const;

/** A labelled slider row with its value at the end, as the settings shipped it. */
function sliderRow(id: string, label: string, value: number, step: number, shown: (v: number) => string, phone: boolean): number {
  let out = value;
  og.ui.row(id, { width: { kind: "grow" }, gap: S.md, cross: "center" }, () => {
    og.ui.label("Label", label, { width: { kind: "fixed", px: phone ? 96 : 140 }, nowrap: true });
    out = og.ui.slider("Slider", value, { step }).value;
    og.ui.label("Value", shown(out), { width: { kind: "fixed", px: 48 }, align: "end", font: "mono", color: { token: "inkMuted" }, nowrap: true });
  });
  return out;
}

/**
 * The settings as they shipped: Sound (the three volumes and Mute), Display (the quality, the frame rate, reduced
 * motion and high contrast), Controls (sensitivity, inverted look and vibration) and Language, as tabs over a body of
 * fixed height (a select on a phone). The quality, the frame rate and the controls are shown at their defaults and
 * kept by nothing, as they always were: Brasswick has none of them.
 */
function settingsPage(world: World, p: PrefsV, at: number): { tab: number; s: Settings } {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const labels = SECTIONS.map((k) => say(world, k));
  const section = say(world, "settings.section");
  const tab = phone
    ? og.ui.select("Section", labels, at, { width: { kind: "grow" }, label: section }).selected
    : og.ui.tabs("Section", labels, at, { fill: true, width: { kind: "grow" }, label: section }).selected;
  const s: Settings = {};
  const pct = (v: number): string => `${Math.round(v * 100)}%`;
  og.ui.column("Body", { width: { kind: "grow" }, gap: S.sm, height: { kind: "fixed", px: phone ? 300 : 260 } }, () => {
    switch (SECTIONS[tab]) {
      case "settings.audio": {
        const volume = (id: string, key: string, value: number): number => sliderRow(id, say(world, key), value, 0.05, pct, phone);
        s.masterVolume = volume("Master", "settings.master", p.master);
        s.musicVolume = volume("Music", "settings.music", p.music);
        s.sfxVolume = volume("Effects", "settings.effects", p.effects);
        s.muted = og.ui.toggle("Mute", p.muted, say(world, "settings.mute")).on;
        break;
      }
      case "settings.graphics": {
        const quality = say(world, "settings.quality");
        og.ui.label("QualityLabel", quality, { step: "meta", color: { token: "inkMuted" } });
        og.ui.radio("Quality", [say(world, "settings.low"), say(world, "settings.medium"), say(world, "settings.high")], 1, { horizontal: !phone, label: quality });
        og.ui.toggle("Fps", false, say(world, "settings.fps"));
        s.reducedMotion = og.ui.toggle("Motion", p.reducedMotion, say(world, "settings.motion")).on;
        s.highContrast = og.ui.toggle("Contrast", p.highContrast, say(world, "settings.contrast")).on;
        break;
      }
      case "settings.controls": {
        sliderRow("Sensitivity", say(world, "settings.sensitivity"), 1, 0.1, (v) => v.toFixed(1), phone);
        og.ui.checkbox("Invert", false, say(world, "settings.invert"));
        og.ui.checkbox("Vibration", true, say(world, "settings.vibration"));
        break;
      }
      default: {
        const languages = say(world, "settings.languages");
        const r = og.ui.radio("Languages", LANGUAGES.map((l) => say(world, l.key)), p.language, { label: languages });
        s.language = LANGUAGES[r.selected]?.code;
        break;
      }
    }
  });
  return { tab, s };
}

function settings(world: World, g: GameV, p: PrefsV): void {
  // assets/ui/settings.ogt: the title and Done; its Page slot the sections and their rows, every change applied at
  // once (src/prefs.ts).
  const asked = screen(world, "settings", g.settingsOpen);
  // The section shown, kept in og.ui's memory between openings (asked every tick, so it is kept).
  const at = og.ui.memory("settingsTab", 0);
  if (!g.settingsOpen) return;
  if (og.ui.openSlot("settings/Page")) {
    const { tab, s } = settingsPage(world, p, at);
    og.ui.close();
    og.ui.remember("settingsTab", tab);
    const before = settingsOf(p);
    if (Object.entries(s).some(([k, v]) => before[k as keyof Settings] !== v)) takeSettings(world, p, s);
  }
  if (asked.includes("done")) {
    g.settingsOpen = false;
    g.settingsNow = true;
  }
}

/** The results' count-up: 0.9 s from their opening, ease-out cubic. */
const COUNT_SECONDS = 0.9;

/**
 * The stars as the results shipped: landing one by one, k 0.3 s after k-1 from the opening, each scaling in, an
 * earned one gold with a burst, the rest in the control ink.
 */
function resultStars(earned: number, max: number, since: number): void {
  const size = 44;
  og.ui.row("Stars", { gap: S.sm, cross: "center" }, () => {
    for (let k = 0; k < max; k++) {
      const shown = og.ui.seconds() - since >= 0.3 * (k + 1);
      const on = k < earned;
      og.ui.transition(og.ui.idi("Star", k), shown, { kind: "scale", instant: false }, () => {
        og.ui.icon("Mark", "star", { size, tint: { token: on ? "gold" : "controlStrong" } });
        if (on) og.ui.burst({ delayMs: 120 });
      });
      if (!shown) og.ui.spacer(og.ui.idi("Gap", k), { kind: "fixed", px: size });
    }
  });
}

function results(world: World, g: GameV): void {
  const n = g.level;
  const b = bestOf(g, n);
  // When they opened, in og.ui's seconds: the count-up and the stars run from there (dropped when the results go).
  let since = og.ui.memory("resultsSince", -1);
  if (since < 0) since = og.ui.seconds();
  og.ui.remember("resultsSince", since);
  const k = Math.min(1, (og.ui.seconds() - since) / COUNT_SECONDS);
  const best = Math.max(b.score, g.newBest ? g.score : 0);
  const asked = screen(world, "results", true, {
    title: say(world, n + 1 < LEVELS ? "results.title" : "results.end", { level: n + 1 }),
    score: k >= 1 ? g.score : Math.round(g.score * (1 - (1 - k) ** 3)),
    best: say(world, "results.bestScore", { best: String(best).replace(/\B(?=(\d{3})+(?!\d))/g, ",") }),
    newBest: g.newBest,
    landed: k >= 1,
    hasNext: n + 1 < LEVELS,
    stats: [
      { label: say(world, "results.gears"), value: say(world, "results.gearsOf", { n: g.gears, of: GEARS[n] }) },
      { label: say(world, "results.time"), value: clock(g.time) },
      { label: say(world, "results.best"), value: clock(b.time) },
      { label: say(world, "results.falls"), value: `${g.deaths}` },
    ],
  });
  if (og.ui.openSlot("results/Stars")) {
    resultStars(g.stars, 3, since);
    og.ui.close();
  }
  const chosen = asked.find((e) => e === "next" || e === "retry" || e === "menu") ?? null;
  switch (chosen) {
    case "next":
      startLevel(world, g, n + 1);
      break;
    case "retry":
      startLevel(world, g, n);
      break;
    case "menu":
      toTitle(world, g);
      g.levelsOpen = true;
      break;
  }
}

export const Ui = defineSystem(
  "Ui",
  { reads: [og.Input, og.Gamepad, Tuning, Zone], writes: [Game, Prefs, og.Lighting2D, og.Audio, og.Transform] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const p = must(world.resource(Prefs));
    const t = must(world.resource(Tuning));
    const pad = must(world.resource(og.Gamepad));

    const held = g.phase === Phase.Play && (g.paused || g.settingsOpen);
    og.time.setScale(world, held ? 0 : 1);
    if (g.phase === Phase.Play && !held && g.exiting === 0 && g.hurt === 0) g.time += time.scaledDt;
    if (g.exiting >= t.exitTicks && g.phase === Phase.Play) finishLevel(world, g);

    const want = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "results";
    const shown = og.ui.screenTransition("Screens", want, { kind: p.reducedMotion ? "fade" : "iris" }).shown;
    if (shown === "title") {
      title(world, g);
      levels(world, g);
    } else if (shown === "play") {
      hud(world, g);
      pause(world, g);
    } else results(world, g);
    // The screen documents of the screens not shown leave the layer stack.
    if (shown !== "title") {
      screen(world, "main-menu", false);
      screen(world, "levels", false);
    }
    if (shown !== "play") screen(world, "pause", false);
    if (shown !== "results") screen(world, "results", false);
    settings(world, g, p);

    // Pause: Escape or Back with nothing open, or the pad's Start. og.ui.back is asked after the dialogs are built:
    // asked before them it takes their Back, and pause and the settings never close on Escape (FINDINGS.md #9).
    if (g.phase === Phase.Play && g.exiting === 0 && !g.settingsOpen) {
      const start = pad.connected(0) && pad.pressed("start", 0);
      if (!g.paused && (og.ui.back() || start)) g.paused = true;
      else if (g.paused && start) g.paused = false;
    }

    // Gameplay keys move Wick: the HUD turns focus navigation off, the dialogs back on (og-ui §4).
    if (g.phase === Phase.Play && !g.paused && !g.settingsOpen) og.ui.setNavigation(false);

    // Accessibility for the playtests: no focusable control without a name.
    if (time.tick % 30 === 0) {
      const tree = og.ui.a11y();
      g.a11yNodes = tree.nodes.length;
      g.a11yUnnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "").length;
    }
    void intent;
  },
);
