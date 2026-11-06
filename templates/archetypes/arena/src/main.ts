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
import { Coin, COIN, CoinDrop, Duel, facts, Feedback, Fighter, FIGHTER, HEIGHT, Hud, Move, must, SeatChanges, setBot, Sound, Sparks, Submit, Tuning, Voice, Voices, WIDTH } from "./game.ts";
import { Seats, setupSeats } from "./seats.ts";

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
  // Two seats (src/creation.ts): lockstep over a room when online, both on this device when local.
  setupSeats(world);

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

  // The feedback (src/game.ts Feedback, Voices): one voice per sound, the coin's sparks.
  for (const [path, bit, volume] of [
    ["audio/coin.ogt", Sound.Coin, 0.7],
    ["audio/dash.ogt", Sound.Dash, 0.45],
    ["audio/win.ogt", Sound.Win, 0.8],
  ] as const) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: must(og.load.sound(world, path)), playing: false, bus: "sfx", volume, priority: 200 }));
    must(world.insert(v, Voice, { bit }));
  }
  const sparks = must(world.spawn({ name: "sparks", position: { x: WIDTH / 2, y: HEIGHT / 2, z: 2 } }));
  must(world.insert(sparks, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/coin.ogt")), seed: 7 }));
  must(world.insert(sparks, Sparks));

}

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/arena.ogt",
  resources: [Duel, Tuning],
  systems: [Seats, Submit, SeatChanges, Move, CoinDrop, Feedback, Voices, Hud],
  start,
  commands: {
    /** `bot:<n>`: this peer's input comes from scripted bot n (0 hands it back to the keys). */
    bot: (_world, args) => setBot(Number(args[0]), args[1] === "tick"),
    /** `rollback:<maxPrediction>`: the room runs in rollback instead of lockstep. */
    /** `stall:<ms>`: og.Lockstep.stallTimeout, 0 never dropping a silent peer. */
    stall: (world, args) => {
      const ms = Number(args[0]);
      if (!Number.isInteger(ms) || ms < 0) throw new Error("write stall:<ms>, 0 or more");
      must(world.resource(og.Lockstep)).stallTimeout = ms;
    },
    rollback: (world, args) => {
      const n = Number(args[0] ?? 8);
      if (!Number.isInteger(n) || n < 1 || n > 30) throw new Error("write rollback:<maxPrediction>, 1 to 30 ticks");
      const lock = must(world.resource(og.Lockstep));
      lock.mode = "rollback";
      lock.maxPrediction = n;
    },
    /** `quick[:<mode>[:<timeoutMs>]]`: quick play, matched with another engine asking for the same mode. */
    quick: (world, args) => {
      const timeoutMs = Number(args[1] ?? 30000);
      if (!Number.isInteger(timeoutMs) || timeoutMs < 0) throw new Error("write quick:<mode>:<timeoutMs>, 0 waiting until roomLeave");
      must(og.platform.quickPlay(world, { mode: args[0] || "duel", players: 2, timeoutMs }));
    },
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
    bot: { usage: "<n>[:tick]", doc: "Drive this peer's player by scripted bot n (1, 2, ...); 0 gives it back to the keys. With :tick the bot plays its seat by the tick alone, not the world (still for the first 60 ticks), so every session gets the same inputs." },
    rollback: { usage: "<maxPrediction>", doc: "Run the room in rollback, predicting up to maxPrediction ticks (1 to 30), instead of lockstep." },
    stall: { usage: "<ms>", doc: "Drop a peer silent this long (og.Lockstep.stallTimeout; 0 never drops)." },
    quick: { usage: "[<mode>[:<timeoutMs>]]", doc: "Quick play: wait to be matched with another engine asking for the same mode (duel by default; natively `og run . --room ws://127.0.0.1:47900/` beside `og relay`), 30 s unless timeoutMs says otherwise." },
    warp: { usage: "<player>:<x>:<y>", doc: "Put fighter 0 (Red) or 1 (Blue) at a point of the floor." },
  },
  facts,
});
