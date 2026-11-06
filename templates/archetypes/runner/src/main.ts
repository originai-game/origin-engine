// Rooftop Run, the runner archetype (2D profile): a courier runs the roofs on
// her own; a tap is a hop over a crate, a hold is a full jump over a fence,
// coins hang in the arcs. The run speeds up until a crash ends it.
//
// Every courier is a seat (src/seats.ts): its og.PlayerInput is all the
// systems read, so the run plays alone, two on one device or two over a
// room in lockstep, as src/creation.ts says. Two seats race on one course:
// seat 1's courier runs a step behind in a cool ink, on the couriers' own
// collider layer so they pass through each other, over the same crates and
// fences; a courier that crashes stops, the round goes on while the other
// runs, and the card names the one who ran farther. One camera shows both.
//
// scenes/main.ogt places the runner (with its Tuning); `start` below sets up
// the orthographic camera, then spawns the world: the sky and the moon, the
// clouds, the city's three tilemap strips (far, near and the roofs, each
// scrolling at its own parallax and repeating), and the pools of crates,
// fences, coins and dust puffs that Spawn and Dust put in play. The systems
// are in src/systems.ts, the components in src/components.ts, the art in
// src/art.ts.

import { og, SHAPE_BOX } from "@origingame/engine";
import type { Entity, FlipbookInfo, TilemapInfo, World } from "@origingame/engine";
import { COURIER, PROPS, loadArt } from "./art.ts";
import { Actor, Hud, Kind, Phase, Race, Run, Sfx, Sound, Sounds, Tuning } from "./components.ts";
import { bit, LOCAL, SEATS, Seats, setBot, setupSeats, useTouchControls } from "./seats.ts";
import type { TouchInput } from "./seats.ts";
import { Animate, Collide, Director, Dust, GROUND_Y, homeX, HudText, Jump, Records, REST_Y, RIVAL, Riders, RUNNER_SIZE, RUNNER_X, SCENERY_SPAN, Scroll, Spawn, Voices } from "./systems.ts";

/** Pool sizes: how many of each can be in play at once. */
const POOLS = { crate: 4, fence: 3, coin: 16, dust: 4 } as const;

/** The visible width the camera keeps at least (portrait screens get a taller view). */
const MIN_VIEW_WIDTH = 14;
/** The visible height on a landscape screen. */
const VIEW_HEIGHT = 10;
/** The lowest visible y: the roofs fill down to here. */
const VIEW_BOTTOM = -1.5;

/** The couriers' collider layer, which no courier's mask takes: racing couriers pass through each other. */
const COURIERS = 8;

/** Sprite layers, back to front (the HUD's text is on 10, floating text on 100). */
const LAYER = { sky: -40, moon: -38, cloud: -36, far: -30, near: -20, roofs: -10, obstacle: 0, coin: 1, dust: 1, runner: 2 } as const;

/** What a row draws: a flipbook clip, a plain texture or a solid colour. */
type Look =
  | { readonly sheet: FlipbookInfo; readonly clip: number }
  | { readonly texture: number }
  | { readonly color: { r: number; g: number; b: number } };

interface ActorDesc {
  readonly name?: string;
  readonly kind: number;
  readonly look: Look;
  readonly layer: number;
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
  readonly hit?: { w: number; h: number };
  readonly parallax?: number;
  readonly wrap?: number;
  readonly opacity?: number;
}

function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

/** The components every actor row carries (src/components.ts says why Run and Tuning are on each). */
function rowComponents(world: World, e: Entity, actor: Partial<{ kind: number; w: number; h: number; parallax: number; wrap: number }>): void {
  must(world.insert(e, Actor, actor));
  must(world.insert(e, Run));
  must(world.insert(e, Tuning));
}

/** Spawn one sprite row: og.Transform, og.Sprite (with an og.Flipbook for a sheet's clip), Actor, Run, Tuning. */
function spawnActor(world: World, d: ActorDesc): Entity {
  const e = world.spawn({ name: d.name ?? "", position: { x: d.x, y: d.y, z: 0 } });
  if (!e.ok) throw new Error(e.reason);
  const look = d.look;
  const texture = "sheet" in look ? look.sheet.texture : "texture" in look ? look.texture : 0;
  must(
    world.insert(e.value, og.Sprite, {
      texture: texture as never,
      width: d.w,
      height: d.h,
      layer: d.layer,
      opacity: d.opacity ?? 1,
      ...("color" in look ? { color: look.color } : {}),
    }),
  );
  if ("sheet" in look) must(world.insert(e.value, og.Flipbook, { sheet: look.sheet.sheet, clip: look.clip }));
  rowComponents(world, e.value, { kind: d.kind, w: d.hit?.w ?? d.w, h: d.hit?.h ?? d.h, parallax: d.parallax ?? 1, wrap: d.wrap ?? 0 });
  return e.value;
}

/** Spawn a city strip: a tilemap whose top edge sits at `top`, scrolling at `parallax` and wrapping by its width (it repeats in x). */
function spawnStrip(world: World, name: string, map: TilemapInfo, top: number, layer: number, parallax: number): Entity {
  const span = map.width * map.tileWidth;
  const e = world.spawn({ name, position: { x: -span / 2, y: top, z: 0 } });
  if (!e.ok) throw new Error(e.reason);
  must(world.insert(e.value, og.Tilemap, { map: map.map, layer, collision: false }));
  rowComponents(world, e.value, { kind: Kind.City, w: 0, h: 0, parallax, wrap: span });
  return e.value;
}

/**
 * The touch controls: a tap is a hop, a hold a full jump. Alone (and in a
 * room) the whole screen is the Jump; two on one device each hold a large
 * Jump button in their own bottom corner (og.ui's action buttons follow
 * each finger, so both thumbs press at once).
 */
function touchJump(seat: number): TouchInput {
  const out = { x: 0, y: 0, lx: 0, ly: 0, buttons: 0 };
  if (LOCAL) {
    og.ui.anchor(`TouchJump${seat}`, { anchor: seat === 0 ? "bottomLeft" : "bottomRight" }, () => {
      if (og.ui.actionButton("Jump", "Jump", { size: 120, primary: true }).pressed) out.buttons |= bit("jump");
    });
    return out;
  }
  const full = { kind: "percent", fraction: 1 } as const;
  og.ui.anchor("TouchJump0", { anchor: "topLeft", margin: 0, width: full, height: full }, () => {
    og.ui.open("Area", { width: full, height: full, interactive: true });
    if (og.ui.close().pressed) out.buttons |= bit("jump");
  });
  return out;
}

function start(world: World): void {
  setupSeats(world);
  useTouchControls(touchJump);
  const art = loadArt(world);
  const font = og.load.font(world, "fonts/ui.ttf");
  if (!font.ok) throw new Error(font.reason);
  // Floating text ("+10") uses the UI's body font.
  must(og.ui.configure(world, { font: font.value }));

  // An orthographic camera over the roofs, at least MIN_VIEW_WIDTH wide.
  const caps = og.render.capabilities(world);
  const aspect = caps.width > 0 && caps.height > 0 ? caps.width / caps.height : 16 / 9;
  const height = Math.max(VIEW_HEIGHT, MIN_VIEW_WIDTH / aspect);
  const halfWidth = (height * aspect) / 2;
  const camera = world.resource(og.Camera);
  if (camera.ok) {
    camera.value.orthographic = true;
    camera.value.height = height;
    camera.value.position = { x: 0, y: VIEW_BOTTOM + height / 2, z: 50 };
    camera.value.background = { r: 0.0042, g: 0.0048, b: 0.018 };
  }

  const top = VIEW_BOTTOM + height;
  spawnActor(world, { name: "sky", kind: Kind.Sky, look: { texture: art.sky as unknown as number }, layer: LAYER.sky, x: 0, y: VIEW_BOTTOM + height / 2, w: Math.max(60, halfWidth * 2 + 1), h: height + 1, parallax: 0 });
  spawnActor(world, { name: "moon", kind: Kind.Moon, look: { sheet: art.props, clip: PROPS.moon }, layer: LAYER.moon, x: halfWidth - 1.9, y: top - 2.3, w: 1.6, h: 1.6, parallax: 0 });
  for (let k = 0; k < 3; k++) {
    spawnActor(world, { kind: Kind.Cloud, look: { sheet: art.props, clip: PROPS.cloud }, layer: LAYER.cloud, x: -SCENERY_SPAN / 2 + (k + 0.5) * (SCENERY_SPAN / 3), y: top - 1.4 - (k % 2) * 1.2, w: 3.2, h: 1.2, parallax: 0.06, wrap: SCENERY_SPAN });
  }

  // The city: far skyline, near buildings, the roofs; each repeats in x and wraps by its own width.
  // Their tops: the skyline and the near buildings stand behind the roof's edge, the roof's top is the ground.
  spawnStrip(world, "far", art.far, -0.4 + art.far.height * art.far.tileHeight, LAYER.far, 0.15);
  spawnStrip(world, "near", art.near, -0.4 + art.near.height * art.near.tileHeight, LAYER.near, 0.45);
  spawnStrip(world, "roofs", art.roofs, GROUND_Y, LAYER.roofs, 1);

  // The pools: parked off screen until Spawn, Dust (or the `place` command) puts them in play.
  const pool = (n: number, name: string, d: Omit<ActorDesc, "x" | "y">) => {
    for (let k = 0; k < n; k++) spawnActor(world, { ...d, name: `${name}-${k}`, x: -1000, y: 0 });
  };
  pool(POOLS.crate, "crate", { kind: Kind.Crate, look: { sheet: art.props, clip: PROPS.crate }, layer: LAYER.obstacle, w: 0.8, h: 0.8, hit: { w: 0.6, h: 0.6 } });
  pool(POOLS.fence, "fence", { kind: Kind.Fence, look: { sheet: art.props, clip: PROPS.fence }, layer: LAYER.obstacle, w: 0.8, h: 1.6, hit: { w: 0.55, h: 1.5 } });
  pool(POOLS.coin, "coin", { kind: Kind.Coin, look: { sheet: art.coin, clip: 0 }, layer: LAYER.coin, w: 0.7, h: 0.7, hit: { w: 0.6, h: 0.6 } });
  pool(POOLS.dust, "dust", { kind: Kind.Dust, look: { color: { r: 0.42, g: 0.36, b: 0.62 } }, layer: LAYER.dust, w: 0.2, h: 0.2, hit: { w: 0, h: 0 }, opacity: 0 });

  // The runner comes from the scene with its Actor and Tuning; give it the courier's sheet, the run, the afterimage and the HUD.
  const runner = world.find("runner");
  if (runner === null) throw new Error("scenes/main.ogt has no entity named runner");
  must(world.insert(runner, og.Sprite, { texture: art.courier.texture, width: RUNNER_SIZE.x, height: RUNNER_SIZE.y, layer: LAYER.runner }));
  must(world.insert(runner, og.Flipbook, { sheet: art.courier.sheet, clip: COURIER.idle }));
  must(world.insert(runner, og.Afterimage, { count: 4, interval: 3, opacity: 0.4, color: { r: 0.35, g: 0.95, b: 0.85 }, active: false }));
  must(world.insert(runner, Run, { joined: true }));
  must(world.insert(runner, Hud, { font: font.value as unknown as number, halfWidth, top, bottom: VIEW_BOTTOM }));
  must(world.set(runner, og.Transform, { position: { x: RUNNER_X, y: 0, z: 0 } }));
  // The body: a box from the feet up that never runs (the world scrolls past
  // it), on og.PlatformerController's jump, moved by its seat's og.PlayerInput;
  // the roof below is one static collider.
  const body = (e: Entity, seat: number) => {
    must(world.insert(e, og.PlayerInput, { player: seat }));
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.5, y: 1.2, z: 1 }, offset: { x: 0, y: 0.6, z: 0 }, layers: COURIERS, mask: ~COURIERS >>> 0 }));
    must(world.insert(e, og.PlatformerController, { moveX: 0, coyoteTime: 0.08, jumpBuffer: 0.1, maxFallSpeed: 30 }));
  };
  body(runner, 0);
  // The second seat's courier, a step behind in its own ink, with the runner's hit box (its row's Tuning is unused: the runner row's tunes both).
  for (let seat = 1; seat < SEATS; seat++) {
    const rival = world.spawn({ name: `runner-${seat}`, position: { x: homeX(seat), y: GROUND_Y, z: 0 } });
    if (!rival.ok) throw new Error(rival.reason);
    const e = rival.value;
    must(world.insert(e, og.Sprite, { texture: art.courier.texture, width: RUNNER_SIZE.x, height: RUNNER_SIZE.y, layer: LAYER.runner, color: RIVAL }));
    must(world.insert(e, og.Flipbook, { sheet: art.courier.sheet, clip: COURIER.idle }));
    must(world.insert(e, og.Afterimage, { count: 4, interval: 3, opacity: 0.4, color: { r: 0.45, g: 0.7, b: 1 }, active: false }));
    must(world.insert(e, Actor, { kind: Kind.Runner, w: 0.5, h: 1.2, parallax: 0 }));
    must(world.insert(e, Run, { seat }));
    must(world.insert(e, Tuning));
    body(e, seat);
  }
  const roof = world.spawn({ name: "roof-solid", position: { x: 0, y: GROUND_Y - 0.5, z: 0 } });
  if (!roof.ok) throw new Error(roof.reason);
  must(world.insert(roof.value, og.Collider, { shape: SHAPE_BOX, size: { x: 80, y: 1, z: 1 } }));

  // The voices: one per sound, played by the Voices system.
  const voices: [string, number, number][] = [
    ["audio/jump.wav", Sound.Jump, 0.45],
    ["audio/coin.wav", Sound.Coin, 0.5],
    ["audio/crash.wav", Sound.Crash, 0.7],
    ["audio/start.wav", Sound.Start, 0.55],
    // A press while the game-over card holds: a quiet tick (og sfx's click).
    ["audio/hold.ogt", Sound.Hold, 0.3],
  ];
  for (const [path, bit, volume] of voices) {
    const sound = og.load.sound(world, path);
    if (!sound.ok) throw new Error(sound.reason);
    const v = world.spawn({ name: `sfx-${bit}` });
    if (!v.ok) throw new Error(v.reason);
    must(world.insert(v.value, og.AudioSource, { sound: sound.value, playing: false, volume }));
    must(world.insert(v.value, Sfx, { bit }));
  }
}


/** The runner's Run, for commands and facts (outside systems). */
function runOf(world: World): { runner: Entity; run: ReturnType<typeof readRun> } | null {
  const runner = world.find("runner");
  if (runner === null) return null;
  return { runner, run: readRun(world, runner) };
}

function readRun(world: World, runner: Entity) {
  const r = world.get(runner, Run);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** `place:<crate|fence|coin>:<x>[:<y>]`: put a pooled obstacle or coin in play at x (and y; default its resting height). */
function placeCommand(world: World, args: readonly string[]): void {
  const kind = { crate: Kind.Crate, fence: Kind.Fence, coin: Kind.Coin }[args[0] ?? ""];
  const x = Number(args[1]);
  if (kind === undefined || !Number.isFinite(x)) throw new Error("write place:<crate|fence|coin>:<x>[:<y>]");
  const y = args[2] === undefined ? REST_Y[kind]! : Number(args[2]);
  const rows = world.query({ with: [Actor] });
  if (!rows.ok) throw new Error(rows.reason);
  for (const a of rows.value.archetypes()) {
    for (let i = 0; i < a.count; i++) {
      const e = a.entity(i);
      const actor = world.get(e, Actor);
      if (!actor.ok || actor.value.kind !== kind || actor.value.active) continue;
      og.tween.cancelEntity(world, e);
      must(world.set(e, Actor, { active: true }));
      must(world.set(e, og.Sprite, { opacity: 1 }));
      must(world.set(e, og.Transform, { position: { x, y, z: 0 }, scale: { x: 1, y: 1, z: 1 } }));
      return;
    }
  }
  throw new Error(`every ${args[0]} is in play; the pool holds ${POOLS[args[0] as "crate" | "fence" | "coin"]}`);
}

export default og.game({
  tick: 60,
  seed: 1,
  scenes: { main: "scenes/main.ogt" },
  resources: [Sounds, Race],
  systems: [Seats, Riders, Records, Director, Jump, Scroll, Spawn, Collide, Voices, Dust, Animate, HudText],
  // MSAA, not the tier's TAA: pixel art stays sharp, with no temporal blur.
  render: { antialias: "msaa4" },
  start,
  commands: {
    place: placeCommand,
    /** `bot:<n>`: this device's seats play a scripted run (0 hands them back to the player). */
    bot: (_world, args) => setBot(Number(args[0])),
  },
  commandHelp: {
    bot: { usage: "<n>", doc: "This device's seats take scripted input by the tick (1, 2, ...); 0 gives them back." },
    place: { usage: "<crate|fence|coin>:<x>[:<y>]", doc: "Put a pooled obstacle or coin in play at x (y defaults to its resting height)." },
  },
  facts: (world) => {
    const r = runOf(world);
    if (r === null) return null;
    const { run } = r;
    return {
      phase: ["title", "running", "over"][run.phase] ?? "unknown",
      score: run.score,
      coins: run.coins,
      distance: Math.round(run.distance * 100) / 100,
      speed: Math.round(run.speed * 100) / 100,
      best: run.best,
      rounds: run.rounds,
      ...(SEATS > 1 ? race(world, run.out) : {}),
    };
  },
});

/** The race's facts: seat 1's metres, who is out, and the winner once the round is over. */
function race(world: World, out: boolean | number): Record<string, unknown> {
  const r = world.resource(Race);
  if (!r.ok) return {};
  return { seats: SEATS, joined2: r.value.joined, racing: r.value.racing, distance2: Math.round(r.value.distance * 100) / 100, out: Boolean(out), out2: r.value.out, winner: r.value.winner };
}

export { Phase };
