// Brasswick's screens on og.ui, dressed by the brasswick skin (the fantasy
// pack recoloured; the look applies it): the title over the first level
// (Continue when the vault slot answered, New game, Levels, Settings), the
// level select (og.ui.levelSelect: stars, best gears and time, locks), the
// HUD (og.ui.platformerHud: gears and the level, touch controls on a
// phone), the pause menu, the settings (og.ui.settingsScreen, every word
// from the string tables, English and Spanish) and the results
// (og.ui.resultsScreen: the score counting up, the stars, the best). The
// screens change through an iris (a fade with reduced motion).
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
import { LANGUAGES, settingsOf, takeSettings } from "./prefs.ts";
import { askContinue } from "./save.ts";

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

const LEVEL_KEYS = ["level.1", "level.2", "level.3"] as const;

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
  const items = [
    { id: "continue", label: g.hasSlot ? say(world, "title.continueAt", { level: g.slotLevel + 1 }) : say(world, "title.continue"), disabled: !g.hasSlot },
    { id: "new", label: say(world, "title.new"), primary: true },
    { id: "levels", label: say(world, "title.levels") },
    { id: "settings", label: say(world, "title.settings") },
  ];
  const r = og.ui.mainMenu("Title", { title: "Brasswick", subtitle: say(world, "title.subtitle"), items, footer: say(world, "title.footer") });
  if (g.levelsOpen || g.settingsOpen) return;
  switch (r.selected) {
    case "continue":
      askContinue(world, g);
      break;
    case "new":
      startLevel(world, g, 0);
      break;
    case "levels":
      g.levelsOpen = true;
      break;
    case "settings":
      g.settingsOpen = true;
      break;
  }
}

function levels(world: World, g: GameV): void {
  if (!g.levelsOpen) return;
  const r = og.ui.levelSelect("Levels", {
    title: say(world, "levels.title"),
    backLabel: say(world, "levels.back"),
    // What a screen reader says for a card's stars and a locked card; the template fills {stars}, {max} and {name}.
    starsLabel: say(world, "levels.stars"),
    lockedLabel: say(world, "levels.locked"),
    columns: 3,
    levels: LEVEL_KEYS.map((key, n) => {
      const b = bestOf(g, n);
      return {
        name: say(world, "levels.card", { n: n + 1, name: say(world, key) }),
        stars: b.stars,
        locked: n >= g.unlocked,
        best: b.time === 0 ? "" : say(world, "levels.best", { gears: b.gears, of: GEARS[n], time: clock(b.time) }),
      };
    }),
  });
  // Its Back button, a tap on its scrim, Escape and the pad's B all answer closed (FINDINGS.md #10, fixed).
  if (r.chosen !== null && r.chosen < g.unlocked) startLevel(world, g, r.chosen);
  else if (r.closed) g.levelsOpen = false;
}

const S = { xs: 4, sm: 8, md: 12 } as const;

function capsule(id: string, build: () => void): void {
  og.ui.row(id, { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "capsule", background: { token: "surface" } }, build);
}

/**
 * The HUD, from widgets: og.ui.platformerHud draws a hearts capsule even with maxLives 0 and a score
 * capsule always, and its level name has no frame (FINDINGS.md #4), so a puzzle game without lives or
 * a score builds its own (og-taste §2: capsules with an icon and a number, in the corners).
 */
function hud(world: World, g: GameV): void {
  // All in the top-right corner: on the published page the shell's Back handle sits over the top-left
  // one, outside og.ui's safe area (FINDINGS.md #15).
  let paused = false;
  og.ui.anchor("Hud", { anchor: "topRight", direction: "row", gap: S.sm, cross: "center" }, () => {
    capsule("Count", () => {
      og.ui.icon("Gear", "gear", { size: 20, tint: { token: "accent" } });
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
  const r = og.ui.pauseMenu("Pause", {
    open: g.paused && !g.settingsOpen,
    title: say(world, "pause.title"),
    items: [
      { id: "resume", label: say(world, "pause.resume"), primary: true },
      { id: "retry", label: say(world, "pause.retry") },
      { id: "restart", label: say(world, "pause.restart") },
      { id: "settings", label: say(world, "pause.settings") },
      { id: "quit", label: say(world, "pause.quit") },
    ],
  });
  switch (r.selected) {
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

function settings(world: World, g: GameV, p: PrefsV): void {
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
    sectionLabel: say(world, "settings.section"),
    languagesLabel: say(world, "settings.languages"),
  });
  if (!g.settingsOpen) return;
  if (r.changed) takeSettings(world, p, r.settings);
  if (r.closed) {
    g.settingsOpen = false;
    g.settingsNow = true;
  }
}

function results(world: World, g: GameV): void {
  const n = g.level;
  const b = bestOf(g, n);
  const actions = [
    ...(n + 1 < LEVELS ? [{ id: "next", label: say(world, "results.next"), primary: true }] : []),
    { id: "retry", label: say(world, "results.retry"), primary: n + 1 >= LEVELS },
    { id: "menu", label: say(world, "results.menu") },
  ];
  const r = og.ui.resultsScreen("Results", {
    open: g.phase === Phase.Results,
    title: say(world, n + 1 < LEVELS ? "results.title" : "results.end", { level: n + 1 }),
    score: g.score,
    best: b.score,
    newBest: g.newBest,
    stars: g.stars,
    maxStars: 3,
    stats: [
      { label: say(world, "results.gears"), value: say(world, "results.gearsOf", { n: g.gears, of: GEARS[n] }) },
      { label: say(world, "results.time"), value: clock(g.time) },
      { label: say(world, "results.best"), value: clock(b.time) },
      { label: say(world, "results.falls"), value: `${g.deaths}` },
    ],
    actions,
    bestLabel: say(world, "results.bestScore"),
    newBestLabel: say(world, "results.newBest"),
    shareLabel: say(world, "results.share"),
  });
  switch (r.selected) {
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

    const screen = g.phase === Phase.Title ? "title" : g.phase === Phase.Play ? "play" : "results";
    const shown = og.ui.screenTransition("Screens", screen, { kind: p.reducedMotion ? "fade" : "iris" }).shown;
    if (shown === "title") {
      title(world, g);
      levels(world, g);
    } else if (shown === "play") {
      hud(world, g);
      pause(world, g);
    } else results(world, g);
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
