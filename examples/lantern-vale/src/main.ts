// Lantern Vale (3D profile): dusk is falling on a valley and four lanterns
// stand dark along its paths. Walk the lamplighter there
// (og.CharacterController on the terrain), hold E by a lantern to light it
// (an og.physics.overlap query finds it) and light all four before night.
// Three slimes patrol: one that sees you bounds after you, and its touch
// costs a heart. Three hearts gone, or night, ends the run; every lantern
// lit wins it. Then the results at dawn, and again.
//
// The look (GameDesc.look, assets/looks/lantern-vale.ogt) sets the frame's
// treatment before `start`; `start` loads the strings, the second look and
// the sounds, builds the vale (src/world.ts) and spawns the hero
// (src/hero.ts), the slimes (src/slimes.ts) and the voices (src/sound.ts).
// The lanterns are src/lanterns.ts, the screens and the clock src/ui.ts,
// the state src/components.ts.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Ending, Game, LANTERN_COUNT, Phase, Prefs, Tuning } from "./components.ts";
import { FEET, HeroMove, spawnHero, View } from "./hero.ts";
import { Lanterns, lightLantern } from "./lanterns.ts";
import { Slimes, spawnSlimes } from "./slimes.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { configureUi, LANGUAGES, nameOf, score, startRun, toTitle, Ui } from "./ui.ts";
import { buildVale, groundY, HOURS, LANTERNS, must, setHour } from "./world.ts";

function start(world: World): void {
  const g = must(world.resource(Game));
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  g.dusk = must(og.load.look(world, "looks/lantern-vale.ogt")) as unknown as number;
  g.dawn = must(og.load.look(world, "looks/dawn.ogt")) as unknown as number;
  configureUi(world, false);
  buildVale(world);
  spawnHero(world);
  spawnSlimes(world);
  spawnVoices(world);
  toTitle(world, g);
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `run`: start a run now, as Set out on the title does. */
function run(world: World): void {
  startRun(world, gameOf(world));
}

/** `warp:<lantern>` or `warp:<x>:<z>`: put the hero beside a lantern (0 to 3) or on the ground at x, z. */
function warp(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  let x: number;
  let z: number;
  if (args.length === 1) {
    const l = LANTERNS[num(args[0], "lantern")];
    if (l === undefined) throw new Error(`lanterns are 0 to ${LANTERNS.length - 1}`);
    x = l.x;
    z = l.z + 1.4;
    g.facing = Math.PI;
  } else {
    x = num(args[0], "x");
    z = num(args[1], "z");
  }
  must(world.set(g.hero as unknown as never, og.Transform, { position: { x, y: groundY(world, g, x, z) + FEET + 0.05, z } }));
}

/** `light:<lantern>`: the lantern catches now, as lighting it does. */
function light(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error("light needs a run (write run first)");
  const i = num(args[0], "lantern");
  if (i < 0 || i >= LANTERN_COUNT) throw new Error(`lanterns are 0 to ${LANTERN_COUNT - 1}`);
  lightLantern(world, g, i);
}

/** `hearts:<n>`: set the hearts left (0 ends the run). */
function hearts(world: World, args: readonly string[]): void {
  gameOf(world).hearts = num(args[0], "n");
}

/** `dusk:<seconds>`: jump the run's clock to this many seconds played (night at Tuning.duskSeconds). */
function dusk(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  g.time = num(args[0], "seconds");
  if (g.phase === Phase.Title) setHour(world, HOURS.title);
}

/** `language:<en|zh-CN>`: switch the language as the settings do. */
function language(world: World, args: readonly string[]): void {
  const p = must(world.resource(Prefs));
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  p.language = at;
  og.i18n.setLocale(world, LANGUAGES[at]!.code);
}

/** `settings`: open the settings, as the title's Settings does. */
function settings(world: World): void {
  gameOf(world).settingsOpen = true;
}

const STREAM = ["unloaded", "queued", "loading", "resident", "failed"] as const;

export default og.game({
  tick: 60,
  seed: 7,
  look: "looks/lantern-vale.ogt",
  scenes: { main: "scenes/main.ogt" },
  resources: [Game, Tuning, Prefs],
  systems: [...og.systems.cameras, Ui, HeroMove, Lanterns, Slimes, View, Voices],
  start,
  commands: { run, warp, light, hearts, dusk, language, settings },
  commandHelp: {
    run: { usage: "", doc: "Start a run now, as Set out does." },
    warp: { usage: "<lantern> | <x>:<z>", doc: "Put the hero beside a lantern (0 to 3; 3 is on the far side), or on the ground at x, z." },
    light: { usage: "<lantern>", doc: "The lantern catches now, as lighting it does (the fourth wins)." },
    hearts: { usage: "<n>", doc: "Set the hearts left (0: the slimes won)." },
    dusk: { usage: "<seconds>", doc: "Jump the run's clock to this many seconds played (night falls at Tuning.duskSeconds)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    settings: { usage: "", doc: "Open the settings, as Settings on the title or the pause menu does." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    const prefs = world.resource(Prefs);
    if (!game.ok || !prefs.ok) return null;
    const g = game.value;
    return {
      phase: ["title", "play", "over"][g.phase],
      ending: ["none", "lit", "night", "caught"][g.ending],
      hp: g.hearts,
      hits: g.hurts,
      lit: g.lit,
      litMask: g.litMask,
      time: Math.round(g.time * 10) / 10,
      hour: Math.round(g.hour * 100) / 100,
      near: g.near,
      kindle: Math.round(g.kindle * 100) / 100,
      speed: Math.round(g.speed * 100) / 100,
      steps: g.steps,
      score: g.ending === Ending.None ? 0 : score(world, g),
      look: og.render.look(world)?.name ?? "",
      far: STREAM[g.far] ?? "unknown",
      farLoads: g.farLoads,
      musicChunks: g.musicChunks,
      rockMoved: Math.round(g.rockMoved * 100) / 100,
      name: nameOf(prefs.value),
      language: LANGUAGES[prefs.value.language]!.code,
      a11yUnnamed: g.a11yUnnamed,
      a11yNodes: g.a11yNodes,
      styled: g.styled,
    };
  },
});
