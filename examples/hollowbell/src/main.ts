// Hollowbell (2D profile): the village bell has lost its three chimes to
// the hollow under the hill. Wren the bell-keeper goes down after them:
// runs and jumps through the caves (og.PlatformerController on a Tiled
// map's colliders and one-way planks), swings a mallet at beetles and moths,
// trades glimmers with Old Burrow the mole, beats the gloam in its hall,
// lights the lanterns that keep the run, and climbs back to ring the bell.
//
// The look (GameDesc.look, assets/looks/hollowbell.ogt: the 16-bit pixel
// preset, the game's own palette) sets the frame before `start`; `start`
// loads the strings, the second look and the art, builds the hollow from
// the map (src/world.ts) and spawns the voices (src/sound.ts). The hero is
// src/hero.ts, the creatures src/enemies.ts, the things src/props.ts, the
// screens src/ui.ts, the saves src/save.ts, the state src/components.ts.

import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { CHIMES, Game, HEARTS, Phase, Prefs, Tuning } from "./components.ts";
import { Enemies, touchHero } from "./enemies.ts";
import { Control } from "./hero.ts";
import { Areas, Backdrop, Bells, Effects, Flickers, Hazards, Lanterns, Lifts, Moles, Pickups, Stones } from "./props.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { continueRun, LANGUAGES, newRun, setLanguage, setName, toTitle, Ui } from "./ui.ts";
import { buildHollow, loadArt, must } from "./world.ts";

function start(world: World): void {
  const g = must(world.resource(Game));
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  g.mainLook = must(og.load.look(world, "looks/hollowbell.ogt")) as unknown as number;
  g.dawnLook = must(og.load.look(world, "looks/dawn.ogt")) as unknown as number;
  // One handle on Game for the whole of start, passed down (FINDINGS.md #6).
  buildHollow(world, loadArt(world), g);
  spawnVoices(world, g);
  toTitle(world, g);
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || s === "" || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

function needRun(world: World, what: string) {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error(`${what} needs a run (write play first)`);
  return g;
}

/** `play`: a new run now, as Begin in the name dialog does. */
function play(world: World): void {
  newRun(world, gameOf(world));
}

/** `continue`: the saved run, as Continue on the title does. */
function resume(world: World): void {
  if (!continueRun(world, gameOf(world), must(world.resource(Prefs)))) throw new Error("no save has answered yet: continue needs a save in the store");
}

/** `warp:<x>:<y>` or `warp:<entity>[:<dx>[:<dy>]]`: put Wren's feet at a world point, or by a named entity (1.5 m left of it unless dx, dy say). */
function warp(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  let x: number;
  let y: number;
  if (Number.isFinite(Number(args[0])) && args[0] !== "") {
    x = num(args[0], "x");
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
}

/** `give:<chimes|glimmers|wick|cloak>[:<n>]`: progress at once (chimes:<n> takes the first n). */
function give(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const n = args[1] === undefined ? 1 : num(args[1], "n");
  switch (args[0]) {
    case "chimes":
      g.chimes = Math.min(CHIMES, n);
      g.chimeMask = (1 << g.chimes) - 1;
      break;
    case "glimmers":
      g.glimmers += n;
      break;
    case "wick":
      g.wick = true;
      break;
    case "cloak":
      g.cloak = true;
      break;
    default:
      throw new Error("write give:<chimes|glimmers|wick|cloak>[:<n>]");
  }
}

/** `hearts:<n>`: set the hearts; fewer than now lands as a beetle's touch from behind. */
function hearts(world: World, args: readonly string[]): void {
  const g = needRun(world, "hearts");
  const n = num(args[0], "n");
  while (g.hearts > n && g.hearts > 0) {
    g.invulnerable = 0;
    touchHero(world, g, g.heroX - g.facing);
  }
  g.hearts = Math.min(HEARTS, n);
}

/** `ring`: ring the bell now (the three chimes needed). */
function ring(world: World): void {
  const g = needRun(world, "ring");
  if (g.chimes < CHIMES) throw new Error(`the bell needs ${CHIMES} chimes, the run has ${g.chimes} (give:chimes:3)`);
  g.ringing = 1;
}

/** `talk`: open Old Burrow's talk, as Up beside him does. */
function talk(world: World): void {
  const g = needRun(world, "talk");
  g.talking = true;
  g.talkIndex = 0;
  g.talks += 1;
}

/** `save`: write the run to the store now, as a lantern does. */
function save(world: World): void {
  needRun(world, "save").saveNow = true;
}

/** `language:<en|zh-CN>`: switch the language as the settings do. */
function language(world: World, args: readonly string[]): void {
  const code = args[0] ?? "";
  if (!LANGUAGES.some((l) => l.code === code)) throw new Error("write language:<en|zh-CN>");
  setLanguage(world, must(world.resource(Prefs)), code);
}

/** `settings`: open the settings, as Settings on the title or the pause menu does. */
function settings(world: World): void {
  gameOf(world).settingsOpen = true;
}

/** `name:<text>`: the bell-keeper's name, as typed in the name dialog. */
function name(world: World, args: readonly string[]): void {
  setName(must(world.resource(Prefs)), args.join(":"));
}

export default og.game({
  look: "looks/hollowbell.ogt",
  resources: [Game, Tuning, Prefs],
  systems: [...og.systems.cameras, Ui, Control, Enemies, Pickups, Lanterns, Bells, Moles, Lifts, Stones, Hazards, Areas, Flickers, Backdrop, Effects, Voices],
  // The pixel look: no antialiasing (the look says so too).
  render: { antialias: "off" },
  start,
  commands: { play, continue: resume, warp, give, hearts, ring, talk, save, language, settings, name },
  commandHelp: {
    play: { usage: "", doc: "Start a new run now, as Begin in the name dialog does." },
    continue: { usage: "", doc: "Continue the saved run, as Continue on the title does." },
    warp: { usage: "<x>:<y> | <entity>[:<dx>[:<dy>]]", doc: "Put Wren's feet at a world point, or by a named entity (lantern-1, mole, bell, lift, gloam): 1.5 m left of it unless dx, dy say." },
    give: { usage: "<chimes|glimmers|wick|cloak>[:<n>]", doc: "Progress at once: the first n chimes, n glimmers, the wick, the cloak." },
    hearts: { usage: "<n>", doc: "Set the hearts; losing them lands as a beetle's touch does." },
    ring: { usage: "", doc: "Ring the bell now (the three chimes needed)." },
    talk: { usage: "", doc: "Open Old Burrow's talk, as Up beside him does." },
    save: { usage: "", doc: "Write the run to the store now, as lighting a lantern does." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    settings: { usage: "", doc: "Open the settings." },
    name: { usage: "<text>", doc: "Set the bell-keeper's name (the save slot's)." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    const prefs = world.resource(Prefs);
    if (!game.ok || !prefs.ok) return null;
    const g = game.value;
    return {
      phase: ["title", "play", "over"][g.phase],
      hp: g.hearts,
      hits: g.hits,
      kills: g.kills,
      hurts: g.hurts,
      chimes: g.chimes,
      glimmers: g.glimmers,
      x: Math.round(g.heroX * 100) / 100,
      y: Math.round(g.heroY * 100) / 100,
      grounded: g.grounded,
      area: g.area,
      ambient: Math.round(g.ambient * 100) / 100,
      light: Math.round(g.light * 100) / 100,
      boss: g.bossHp,
      bossDown: g.bossDown,
      wick: g.wick,
      cloak: g.cloak,
      checkpoint: g.checkpoint,
      saves: g.saves,
      loads: g.loads,
      hasSave: g.hasSave,
      score: g.score,
      best: g.best,
      look: og.render.look(world)?.name ?? "",
      name: prefs.value.name,
      language: LANGUAGES[prefs.value.language]!.code,
      a11yUnnamed: g.a11yUnnamed,
      a11yNodes: g.a11yNodes,
    };
  },
});

export type { Entity };
