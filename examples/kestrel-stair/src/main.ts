// The Kestrel Stair (3D profile): a third-person climb of a ruined
// hillside monastery at the end of a long afternoon. The courier Ilse
// carries the Abbess's letter up the stair to the bell-keeper on the
// summit: through the lower court (thorn imps), up the ramp to the crypt
// terrace (a crypt with the first sigil, a stone warden carrying the third,
// the ledges to the second), the stairs to the high terrace, the summit
// gate that three sigils raise, and the bell. Before nightfall at 19:30
// (five minutes of play), or the climb is lost; strength gone, too.
//
// The look (GameDesc.look, assets/looks/kestrel.ogt, extending the
// watercolour-wash preset) is applied before `start`; `start` loads the
// strings, the second look and the navmesh, builds the stair
// (src/world.ts) and spawns the courier (src/hero.ts), the foes
// (src/foes.ts), the voices (src/sound.ts) and the listener (src/view.ts).
// The screens and the clock are src/ui.ts, the platform src/save.ts, the
// state src/components.ts, the layout (shared with tools/) src/layout.ts.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Ending, Foe, Game, Kind, Phase, Prefs, Scene, Tuning } from "./components.ts";
import { alert, bite, Foes, slam, spawnFoes } from "./foes.ts";
import { Fx } from "./fx.ts";
import { HeroMove, hurtHero, placeHero, spawnHero } from "./hero.ts";
import { BELL, CRYPT, FOES, GATE, HERBS, LEDGES, SIGILS, WAYSTONES } from "./layout.ts";
import { Pickups, raiseGate, rest, ringBell, takeSigil } from "./pickups.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { LANGUAGES, setLanguage, startRun, Ui } from "./ui.ts";
import { spawnListener, View } from "./view.ts";
import { buildStair, gateObstacle, groundAt, must } from "./world.ts";

function start(world: World): void {
  const g = must(world.resource(Game));
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  g.lookStair = must(og.load.look(world, "looks/kestrel.ogt")) as unknown as number;
  g.lookSummit = must(og.load.look(world, "looks/summit.ogt")) as unknown as number;
  buildStair(world, g);
  g.nav = must(og.nav.load(world, "nav/stair.ogt")).mesh as unknown as number;
  gateObstacle(world, g, true);
  spawnHero(world, g);
  spawnFoes(world, g);
  spawnVoices(world, g);
  spawnListener(world);
  g.phase = Phase.Title;
  g.start = true;
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || s === "" || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `play[:<chapter>]`: a run from chapter 0 to 3 now, as New journey or Chapters does; `play:<c>:quick` skips the story. */
function play(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const c = args[0] === undefined ? 0 : num(args[0], "chapter");
  if (c < 0 || c > 3) throw new Error("chapters are 0 to 3");
  startRun(world, g, c);
  if (args[1] === "quick") {
    g.scene = Scene.None;
    g.tutorial = false;
  }
}

/** Named places a test can warp to: waystones, sigils, herbs, the crypt, ledges, the gate, the bell, a foe. */
function place(world: World, name: string): { x: number; y: number; z: number; facing: number } {
  const [kind, n] = name.split("-") as [string, string | undefined];
  const i = n === undefined ? 0 : Number(n);
  if (kind === "waystone" && WAYSTONES[i]) return { x: WAYSTONES[i]!.x - 1.2, y: WAYSTONES[i]!.y, z: WAYSTONES[i]!.z - 0.8, facing: Math.PI };
  if (kind === "sigil" && SIGILS[i]) {
    const at = must(world.get(world.find(`sigil-${i}`)!, og.Transform)).position;
    return { x: at.x + 1.2, y: at.y - 0.8, z: at.z, facing: -Math.PI / 2 };
  }
  if (kind === "herb" && HERBS[i]) return { x: HERBS[i]!.x + 1, y: HERBS[i]!.y, z: HERBS[i]!.z, facing: -Math.PI / 2 };
  if (kind === "crypt") return { x: CRYPT.max[0] + 1.5, y: 4, z: (CRYPT.door.z0 + CRYPT.door.z1) / 2, facing: -Math.PI / 2 };
  if (kind === "ledge" && LEDGES[i]) return { x: LEDGES[i]!.x, y: LEDGES[i]!.top, z: LEDGES[i]!.z, facing: Math.PI };
  if (kind === "gate") return { x: 0, y: GATE.min[1], z: GATE.max[2] + 2.2, facing: Math.PI };
  if (kind === "bell") return { x: BELL.x, y: BELL.y, z: BELL.z + 2.2, facing: Math.PI };
  const foe = FOES.find((f) => f.name === name);
  if (foe) {
    const at = must(world.get(world.find(name)!, og.Transform)).position;
    return { x: at.x, y: at.y, z: at.z + 2.4, facing: Math.PI };
  }
  throw new Error(`no place ${name}: waystone-<0..3>, sigil-<0..2>, herb-<0..3>, crypt, ledge-<0..4>, gate, bell or a foe (${FOES.map((f) => f.name).join(", ")})`);
}

/** `warp:<place>[:<dx>:<dz>]` or `warp:<x>:<z>[:<y above>]`: the courier there, still, facing the way the place says. */
function warp(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  if (Number.isFinite(Number(args[0]))) {
    const x = num(args[0], "x");
    const z = num(args[1], "z");
    const y = groundAt(world, x, args[2] === undefined ? 14 : num(args[2], "y"), z);
    placeHero(world, g, x, y, z, Math.PI);
    return;
  }
  const at = place(world, args[0] ?? "");
  const dx = args[1] === undefined ? 0 : num(args[1], "dx");
  const dz = args[2] === undefined ? 0 : num(args[2], "dz");
  placeHero(world, g, at.x + dx, groundAt(world, at.x + dx, at.y + 0.5, at.z + dz), at.z + dz, at.facing);
}

/** `give:<sigils|herbs>:<n>`: progress at once (sigils as a count, the first n). */
function give(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const n = num(args[1] ?? "1", "n");
  if (args[0] === "sigils") for (let i = 0; i < Math.min(3, n); i++) takeSigil(world, g, i);
  else if (args[0] === "herbs") g.herbs += n;
  else throw new Error("give:<sigils|herbs>:<n>");
}

/** `hp:<n>`: the courier's strength; a loss in a run lands as a hurt does (its hit stop, shake, number and sound). */
function hp(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const n = num(args[0], "strength");
  if (n < g.hp && g.phase === Phase.Play && g.ending === Ending.None) hurtHero(world, g, must(world.resource(Tuning)), g.hp - n, { x: g.heroX, z: g.heroZ - 1 }, false);
  g.hp = n;
}

/** `dusk:<seconds>`: jumps the run's clock to this many seconds played (night at Tuning.duskSeconds). */
function dusk(world: World, args: readonly string[]): void {
  gameOf(world).time = num(args[0], "seconds");
}

/** `rest:<waystone>`: rests there now (strength, the save), wherever the courier is. */
function restAt(world: World, args: readonly string[]): void {
  rest(world, gameOf(world), num(args[0], "waystone"));
}

/** `ring`: the gate raised and the bell rung now (the climb's end). */
function ring(world: World): void {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error("ring needs a run (write play first)");
  raiseGate(world, g);
  ringBell(world, g);
}

/** `language:<en|ja>`: switches the language as the settings do. */
function language(world: World, args: readonly string[]): void {
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|ja>");
  setLanguage(world, must(world.resource(Prefs)), at);
}

/** `open:<settings|bag|chapters|records>`: opens a screen, as its button does. */
function open(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  if (args[0] === "settings") g.settingsOpen = true;
  else if (args[0] === "bag") g.bagOpen = true;
  else if (args[0] === "chapters") g.chaptersOpen = true;
  else if (args[0] === "records") g.recordsOpen = true;
  else throw new Error("open:<settings|bag|chapters|records>");
}


const PHASES = ["title", "play", "ending", "over"];
const ENDINGS = ["none", "rung", "night", "fallen"];
const NEAR = ["none", "sigil", "herb", "waystone", "gate", "bell"];

export default og.game({
  look: "looks/kestrel.ogt",
  render: { antialias: "taa" },
  resources: [Game, Tuning, Prefs],
  systems: [...og.systems.cameras, Ui, HeroMove, Foes, Pickups, View, Fx, Voices],
  start,
  commands: { play, warp, give, hp, dusk, rest: restAt, ring, language, open, bite, alert, slam },
  commandHelp: {
    play: { usage: "[<chapter 0..3>[:quick]]", doc: "A run from that chapter now, as New journey or Chapters does; quick skips the story and the tutorial." },
    warp: { usage: "<place>[:<dx>:<dz>] | <x>:<z>[:<y>]", doc: "The courier at a named place (waystone-N, sigil-N, herb-N, crypt, ledge-N, gate, bell, a foe) or on the ground at x, z (below y)." },
    give: { usage: "<sigils|herbs>:<n>", doc: "Progress at once." },
    hp: { usage: "<n>", doc: "The courier's strength (0 to 100)." },
    dusk: { usage: "<seconds>", doc: "Jump the run's clock (night at Tuning.duskSeconds)." },
    rest: { usage: "<waystone>", doc: "Rest at that waystone now: strength back, the world saved." },
    ring: { usage: "", doc: "Raise the gate and ring the bell now: the climb's end." },
    language: { usage: "<en|ja>", doc: "Switch the language as the settings do." },
    open: { usage: "<settings|bag|chapters|records>", doc: "Open a screen as its button does." },
    bite: { usage: "<foe>", doc: "An imp's bite (og.ai sends the imp's entity; or a foe index 0 to 4)." },
    alert: { usage: "<foe>", doc: "A warden has seen the courier." },
    slam: { usage: "<foe>", doc: "A warden's slam (a long wind-up, a lunge, the hit)." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    const prefs = world.resource(Prefs);
    if (!game.ok || !prefs.ok) return null;
    const g = game.value;
    const sigils = [0, 1, 2].filter((b) => (g.sigils & (1 << b)) !== 0).length;
    const foes: Record<string, unknown> = {};
    for (const f of FOES) {
      const e = world.find(f.name);
      if (e === null) continue;
      const v = world.get(e, Foe);
      if (v.ok) foes[f.name] = { hp: v.value.hp, kind: v.value.kind === Kind.Imp ? "imp" : "warden", down: v.value.dying > 0, bites: v.value.bites };
    }
    return {
      phase: PHASES[g.phase],
      ending: ENDINGS[g.ending],
      chapter: g.chapter,
      hp: Math.round(g.hp),
      hits: g.hits,
      kills: g.kills,
      hurts: g.hurts,
      sigils,
      herbs: g.herbs,
      time: Math.round(g.time * 10) / 10,
      hour: Math.round(g.hour * 100) / 100,
      near: NEAR[g.nearKind],
      x: Math.round(g.heroX * 100) / 100,
      y: Math.round(g.heroY * 100) / 100,
      z: Math.round(g.heroZ * 100) / 100,
      grounded: g.grounded,
      speed: Math.round(g.speed * 100) / 100,
      gateOpen: g.gateOpen,
      inCrypt: g.inCrypt,
      reverb: Math.round(g.reverb * 100) / 100,
      scene: ["none", "intro", "end"][g.scene],
      tutorial: g.tutorial,
      look: og.render.look(world)?.name ?? "",
      saves: g.saves,
      loads: g.loads,
      hasSave: g.hasSave,
      saveBytes: g.saveBytes,
      best: g.best,
      score: g.score,
      rank: g.rank,
      reached: g.chaptersReached,
      stars: g.chapterStars,
      musicChunks: g.musicChunks,
      language: LANGUAGES[prefs.value.language]!.code,
      a11yUnnamed: g.a11yUnnamed,
      a11yNodes: g.a11yNodes,
      foes,
      ended: g.ending !== Ending.None,
    };
  },
});
