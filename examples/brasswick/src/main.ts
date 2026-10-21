// Brasswick (2D profile): a short puzzle-platformer in a lamplit clockwork
// vault. Wick, a small brass automaton with a candle for a wick, climbs out
// through three levels: he runs and jumps (og.PlatformerController: coyote
// time, a jump buffer, wall slides and wall jumps, grates he drops through),
// shoves crates onto pressure plates to open doors and a jointed gate, rides
// a lift over spikes, stomps clockwork ticks, gathers gears, lights lamps
// (the checkpoints, and the save), and reaches each level's hatch.
//
// The look (GameDesc.look, assets/looks/brasswick.ogt: the detailed-pixel
// preset at 360 lines, the brasswick skin) sets the frame before `start`;
// `start` loads the string tables and the art, places the three tilemaps
// and spawns their things (src/level.ts), Wick (src/level.ts, src/hero.ts),
// the voices and the music (src/sound.ts), and shows the title (src/ui.ts).
// The puzzle pieces are src/puzzle.ts, what Wick meets src/actors.ts, the
// saves src/save.ts, the settings src/prefs.ts, the juice src/feel.ts.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Exits, Hazards, Lamps, Pickups, Ticks } from "./actors.ts";
import { Fx, Game, GEARS, LEVELS, Phase, Prefs, Sfx, Tuning } from "./components.ts";
import { Effects, Settle } from "./feel.ts";
import { Clip, Control } from "./hero.ts";
import { buildVault, clip, LEVEL_PITCH, loadArt, must, spawnWick, VIEW_H } from "./level.ts";
import { applyPrefs, LANGUAGES, setLanguage } from "./prefs.ts";
import { Crates, Doors, Gates, Lifts, Plates } from "./puzzle.ts";
import { Answers, askContinue, Saves } from "./save.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { finishLevel, startLevel, toTitle, Ui } from "./ui.ts";

function start(world: World): void {
  const g = must(world.resource(Game));
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const art = loadArt(world);
  // Control picks Wick's clips by index: check the sheet's order matches.
  for (const [name, index] of Object.entries(Clip)) if (clip(art.wick, name) !== index) throw new Error(`sprites/wick.ogt's clip ${name} is not ${index}`);

  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = VIEW_H;
  camera.background = { r: 0.004, g: 0.0035, b: 0.006 };
  camera.position = { x: 15, y: -10, z: 50 };

  const lighting = must(world.resource(og.Lighting2D));
  lighting.enabled = true;
  lighting.minLayer = -100;
  lighting.maxLayer = 50;

  buildVault(world, art, g);
  const l1 = must(world.get(must(nonNull(world.find("l1-spawn"))), og.Transform)).position;
  spawnWick(world, art, l1.x, l1.y);
  spawnVoices(world);
  applyPrefs(world, must(world.resource(Prefs)));
  toTitle(world, g);
}

function nonNull<T>(v: T | null): { ok: true; value: T } | { ok: false; reason: string } {
  return v === null ? { ok: false, reason: "missing" } : { ok: true, value: v };
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || s === "" || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

function needPlay(world: World, what: string) {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error(`${what} needs a level (write play:<1-3> first)`);
  return g;
}

export default og.game({
  look: "looks/brasswick.ogt",
  resources: [Game, Prefs, Tuning, Fx, Sfx],
  systems: [...og.systems.cameras, Answers, Ui, Control, Crates, Plates, Doors, Gates, Lifts, Ticks, Pickups, Lamps, Exits, Hazards, Saves, Effects, Voices, Settle],
  // The pixel look: no antialiasing (the preset says so too).
  render: { antialias: "off" },
  start,
  commands: {
    play: (world, args) => {
      const n = args[0] === undefined ? 1 : num(args[0], "the level");
      if (n < 1 || n > LEVELS) throw new Error(`write play:<1-${LEVELS}>`);
      startLevel(world, gameOf(world), n - 1);
    },
    warp: (world, args) => {
      const g = needPlay(world, "warp");
      let x: number;
      let y: number;
      if (args[0] !== undefined && args[0] !== "" && Number.isFinite(Number(args[0]))) {
        // In the level's own units: x from its map's left edge (each map sits at level * LEVEL_PITCH).
        x = g.level * LEVEL_PITCH + num(args[0], "x");
        y = num(args[1], "y");
      } else {
        const e = world.find(args[0] ?? "");
        if (e === null) throw new Error(`no entity named ${args[0]}: write warp:<x>:<y> or warp:<name>[:<dx>[:<dy>]]`);
        const p = must(world.get(e, og.Transform)).position;
        x = p.x + (args[1] === undefined ? -1.5 : num(args[1], "dx"));
        y = p.y + (args[2] === undefined ? 0 : num(args[2], "dy"));
      }
      g.warp = true;
      g.warpX = x;
      g.warpY = y;
    },
    finish: (world) => {
      const g = needPlay(world, "finish");
      finishLevel(world, g);
    },
    gears: (world, args) => {
      const g = needPlay(world, "gears");
      g.gears = Math.min(GEARS[g.level], num(args[0], "n"));
    },
    hurt: (world) => {
      needPlay(world, "hurt").hurtAsk = true;
    },
    open_pause: (world) => {
      needPlay(world, "pause").paused = true;
    },
    title: (world) => toTitle(world, gameOf(world)),
    levels: (world) => {
      const g = gameOf(world);
      toTitle(world, g);
      g.levelsOpen = true;
    },
    settings: (world) => {
      gameOf(world).settingsOpen = true;
    },
    language: (world, args) => {
      const code = args[0] ?? "";
      if (!LANGUAGES.some((l) => l.code === code)) throw new Error(`write language:<${LANGUAGES.map((l) => l.code).join("|")}>`);
      setLanguage(world, must(world.resource(Prefs)), code);
    },
    unlock: (world, args) => {
      gameOf(world).unlocked = Math.max(1, Math.min(LEVELS, num(args[0], "levels")));
    },
    continue: (world) => {
      const g = gameOf(world);
      if (!g.hasSlot) throw new Error("no vault slot has answered: light a lamp first, or wait for the title's read");
      askContinue(world, g);
    },
  },
  commandHelp: {
    play: { usage: "[1-3]", doc: "Start a level from its spawn now, as the level select does." },
    warp: { usage: "<x>:<y> | <entity>[:<dx>[:<dy>]]", doc: "Put Wick's feet at a point of the level (x from its map's left edge, y up from its top: the floors are negative), or by a named entity (l1-crate-0, l2-door-a, l3-exit): 1.5 left of it unless dx, dy say." },
    finish: { usage: "", doc: "Finish the level now: the results, the progress written." },
    gears: { usage: "<n>", doc: "Set the level's gears taken." },
    hurt: { usage: "", doc: "Hurt Wick now, as a tick or the spikes do." },
    open_pause: { usage: "", doc: "Pause, as Escape does." },
    title: { usage: "", doc: "Back to the title." },
    levels: { usage: "", doc: "The title with the level select open." },
    settings: { usage: "", doc: "Open the settings." },
    language: { usage: "<en|es>", doc: "Switch the language as the settings do." },
    unlock: { usage: "<1-3>", doc: "Unlock the first n levels." },
    continue: { usage: "", doc: "Continue from the vault slot, as Continue on the title does." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    const prefs = world.resource(Prefs);
    if (!game.ok || !prefs.ok) return null;
    const g = game.value;
    return {
      phase: ["title", "play", "results"][g.phase],
      level: g.level + 1,
      gears: g.gears,
      kills: g.stomps,
      hurts: g.deaths,
      time: Math.round(g.time * 100) / 100,
      x: Math.round(g.heroX * 100) / 100,
      y: Math.round(g.heroY * 100) / 100,
      grounded: g.grounded,
      onWall: g.onWall,
      pushing: g.pushing,
      switches: g.switches,
      checkpoint: g.checkpoint,
      paused: g.paused,
      score: g.score,
      stars: g.stars,
      unlocked: g.unlocked,
      saves: g.saves,
      loads: g.loads,
      hasSlot: g.hasSlot,
      language: LANGUAGES[prefs.value.language]!.code,
      a11yUnnamed: g.a11yUnnamed,
    };
  },
});
