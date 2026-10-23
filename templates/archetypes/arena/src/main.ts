// Coin Duel, the arena archetype (2D profile): two players on one floor run
// for one coin; each touch is a point and the coin jumps somewhere else,
// first to five wins the round. It plays alone (the local keys drive Red)
// and over a room in lockstep: `og relay` and two `og run . --room
// ws://127.0.0.1:47900/duel` processes (or two Play pages in one platform
// room) each run the whole world and exchange only their inputs and the
// world's hash (og.net, docs/design.md §2.10).
//
// start turns og.Lockstep on for two players and spawns the two fighters,
// each an og.PlayerInput whose `player` is its seat, and the coin. The
// systems are in src/game.ts: Submit hands this peer's keys to
// og.net.submit, Move and CoinDrop read og.PlayerInput and og.Rng only.

import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { Coin, COIN, CoinDrop, Duel, facts, Fighter, FIGHTER, HEIGHT, Hud, Move, must, setBot, Submit, Tuning, WIDTH } from "./game.ts";

/** The brand ink #14161b, linear, behind the floor. */
const INK = { r: 0.00699, g: 0.00802, b: 0.01096 };
const FLOOR = { r: 0.022, g: 0.026, b: 0.034 };
const LINE = { r: 0.06, g: 0.068, b: 0.085 };
/** Red and Blue, linear. */
const INKS = [
  { r: 0.9, g: 0.12, b: 0.1 },
  { r: 0.1, g: 0.38, b: 0.95 },
];
const GOLD = { r: 1, g: 0.62, b: 0.08 };

function rect(world: World, name: string, x: number, y: number, w: number, h: number, color: { r: number; g: number; b: number }, layer: number, glow = 0): Entity {
  const e = must(world.spawn({ name, position: { x, y, z: 0 } }));
  must(world.insert(e, og.Sprite, { width: w, height: h, color, layer, glow }));
  return e;
}

function start(world: World): void {
  // Lockstep for two: in a room the world waits for both seats' inputs each tick.
  const lock = must(world.resource(og.Lockstep));
  lock.enabled = true;
  lock.players = 2;
  lock.inputDelay = 2;
  lock.hashEvery = 10;

  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = HEIGHT + 2;
  camera.position = { x: WIDTH / 2, y: HEIGHT / 2, z: 50 };
  camera.background = INK;

  rect(world, "floor", WIDTH / 2, HEIGHT / 2, WIDTH, HEIGHT, FLOOR, -10);
  rect(world, "midline", WIDTH / 2, HEIGHT / 2, 0.06, HEIGHT, LINE, -9);

  for (const player of [0, 1]) {
    const e = rect(world, `fighter-${player}`, player === 0 ? 3 : WIDTH - 3, HEIGHT / 2, FIGHTER * 2, FIGHTER * 2, INKS[player]!, 1);
    must(world.insert(e, og.PlayerInput, { player }));
    must(world.insert(e, Fighter, { faceX: player === 0 ? 1 : -1 }));
  }
  const coin = rect(world, "coin", WIDTH / 2, HEIGHT / 2, COIN * 2, COIN * 2, GOLD, 0, 0.6);
  must(world.insert(coin, Coin));

  const font = og.load.font(world, "fonts/ui.ttf");
  if (font.ok) og.ui.configure(world, { font: font.value });
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Duel, Tuning],
  systems: [Submit, Move, CoinDrop, Hud],
  start,
  commands: {
    /** `bot:<n>`: this peer's input comes from scripted bot n (0 hands it back to the keys). */
    bot: (_world, args) => setBot(Number(args[0])),
    /** `warp:<player>:<x>:<y>`: put a fighter there. */
    warp: (world, args) => {
      const e = world.find(`fighter-${Number(args[0])}`);
      const x = Number(args[1]);
      const y = Number(args[2]);
      if (e === null || !Number.isFinite(x) || !Number.isFinite(y)) throw new Error("write warp:<player>:<x>:<y>");
      must(world.set(e, og.Transform, { position: { x, y, z: 0 } }));
    },
  },
  commandHelp: {
    bot: { usage: "<n>", doc: "Drive this peer's player by scripted bot n (1, 2, ...); 0 gives it back to the keys." },
    warp: { usage: "<player>:<x>:<y>", doc: "Put fighter 0 (Red) or 1 (Blue) at a point of the floor." },
  },
  facts,
});
