// Rooftop Run, the runner archetype (2D profile): a courier runs the roofs on
// her own; a tap is a hop over a crate, a hold is a full jump over a fence,
// coins hang in the arcs. The run speeds up until a crash ends it.
//
// scenes/main.ogt places the runner (with its Tuning); `start` below sets up
// the orthographic camera and a dark sun, then spawns the sprites: the sky,
// the moon, clouds and buildings (parallax), the roof tiles, and the pools of
// crates, fences and coins that Spawn puts in play. The systems are in
// src/systems.ts, the components in src/components.ts, the sprite sheet in
// src/art.ts.

import { og } from "@origingame/engine";
import type { Entity, Material, World } from "@origingame/engine";
import { loadSheet } from "./art.ts";
import type { Sheet, SpriteName } from "./art.ts";
import { Actor, Hud, Kind, Phase, Run, Tuning } from "./components.ts";
import { Animate, Collide, Director, HudText, Jump, REST_Y, RUNNER_SIZE, RUNNER_X, SCENERY_SPAN, Scroll, Spawn } from "./systems.ts";

/** Pool sizes: how many of each can be in play at once. */
const POOLS = { crate: 4, fence: 3, coin: 16 } as const;

/** The visible width the camera keeps at least (portrait screens get a taller view). */
const MIN_VIEW_WIDTH = 14;
/** The visible height on a landscape screen. */
const VIEW_HEIGHT = 10;
/** The lowest visible y: the roof tiles fill down to here. */
const VIEW_BOTTOM = -1.5;

interface ActorDesc {
  readonly name?: string;
  readonly kind: number;
  readonly sprite: SpriteName;
  readonly x: number;
  readonly y: number;
  readonly z: number;
  readonly w: number;
  readonly h: number;
  readonly hit?: { w: number; h: number };
  readonly parallax?: number;
  readonly wrap?: number;
  readonly active?: boolean;
}

/** Spawn one actor row with the five components every row carries (src/components.ts). */
function spawnActor(world: World, sheet: Sheet, d: ActorDesc): Entity {
  const e = world.spawn({ name: d.name ?? "", position: { x: d.x, y: d.y, z: d.z }, scale: { x: d.w, y: d.h, z: 1 } });
  if (!e.ok) throw new Error(e.reason);
  const sprite = sheet[d.sprite];
  const active = d.active ?? true;
  must(world.insert(e.value, og.Mesh, { geometry: sprite.geometry, material: sprite.material, visible: active }));
  must(
    world.insert(e.value, Actor, {
      kind: d.kind,
      active,
      w: d.hit?.w ?? d.w,
      h: d.hit?.h ?? d.h,
      parallax: d.parallax ?? 1,
      wrap: d.wrap ?? 0,
      look0: sprite.material,
    }),
  );
  must(world.insert(e.value, Run));
  must(world.insert(e.value, Tuning));
  return e.value;
}

function must(r: { ok: true } | { ok: false; reason: string }): void {
  if (!r.ok) throw new Error(r.reason);
}

function start(world: World): void {
  const sheet = loadSheet(world);
  const font = og.load.font(world, "fonts/ui.ttf");
  if (!font.ok) throw new Error(font.reason);

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
    camera.value.background = { r: 0.0242, g: 0.0185, b: 0.0513 };
  }
  // The sprites are emissive: no environment light, no tone mapping, so the frame shows their colours exactly.
  const settings = world.resource(og.RenderSettings);
  if (settings.ok) {
    settings.value.environmentIntensity = 0;
    settings.value.toneMapped = false;
  }

  // A dark sun replaces the renderer's default one, so no light adds to the sprites.
  const sun = world.spawn({ name: "sun" });
  if (sun.ok) must(world.insert(sun.value, og.DirectionalLight, { intensity: 0, castShadow: false }));

  const top = VIEW_BOTTOM + height;
  spawnActor(world, sheet, { name: "sky", kind: Kind.Sky, sprite: "sky", x: 0, y: VIEW_BOTTOM + height / 2, z: -40, w: Math.max(60, halfWidth * 2 + 1), h: height + 1, parallax: 0 });
  spawnActor(world, sheet, { name: "moon", kind: Kind.Moon, sprite: "moon", x: halfWidth * 0.75, y: top - 2.5, z: -38, w: 2.2, h: 2.2, parallax: 0 });

  // Clouds, buildings and roof tiles wrap over SCENERY_SPAN, whatever the view's width.
  for (let k = 0; k < 3; k++) {
    spawnActor(world, sheet, { kind: Kind.Cloud, sprite: "cloud", x: -SCENERY_SPAN / 2 + (k + 0.5) * (SCENERY_SPAN / 3), y: top - 1.6 - (k % 2) * 1.3, z: -36 + k * 0.1, w: 4, h: 1.5, parallax: 0.08, wrap: SCENERY_SPAN });
  }
  const buildings: SpriteName[] = ["building_a", "building_b", "building_c"];
  const bCount = 12;
  const bStep = SCENERY_SPAN / bCount;
  for (let k = 0; k < bCount; k++) {
    const tall = 2.8 + ((k * 7) % 5) * 0.35;
    spawnActor(world, sheet, {
      kind: Kind.Building,
      sprite: buildings[k % 3]!,
      x: -SCENERY_SPAN / 2 + (k + 0.5) * bStep,
      y: GROUND_TOP + tall / 2 - 0.4,
      z: -30 + (k % 2) * 0.1,
      w: tall / 2,
      h: tall,
      parallax: 0.3,
      wrap: SCENERY_SPAN,
    });
  }
  const tCount = SCENERY_SPAN / 2;
  for (let k = 0; k < tCount; k++) {
    spawnActor(world, sheet, { kind: Kind.Ground, sprite: "ground", x: -SCENERY_SPAN / 2 + 1 + k * 2, y: GROUND_TOP - 0.75, z: -5, w: 2, h: 1.5, wrap: SCENERY_SPAN });
  }

  // The pools: parked off screen until Spawn (or the `place` command) puts them in play.
  const pool = (n: number, kind: number, sprite: SpriteName, w: number, h: number, hit: { w: number; h: number }) => {
    for (let k = 0; k < n; k++) spawnActor(world, sheet, { name: `${sprite}-${k}`, kind, sprite, x: -1000, y: 0, z: kind === Kind.Coin ? 0.5 : 0, w, h, hit, active: false });
  };
  pool(POOLS.crate, Kind.Crate, "crate", 0.8, 0.8, { w: 0.6, h: 0.6 });
  pool(POOLS.fence, Kind.Fence, "fence", 0.8, 1.6, { w: 0.55, h: 1.5 });
  pool(POOLS.coin, Kind.Coin, "coin", 0.7, 0.7, { w: 0.6, h: 0.6 });

  // The runner comes from the scene with its Actor and Tuning; give it the sprites, the run and the HUD.
  const runner = world.find("runner");
  if (runner === null) throw new Error("scenes/main.ogt has no entity named runner");
  const look = (n: SpriteName): Material => sheet[n].material;
  must(world.insert(runner, og.Mesh, { geometry: sheet.runner_idle.geometry, material: look("runner_idle") }));
  must(world.set(runner, Actor, { look0: look("runner_idle"), look1: look("runner_run_a"), look2: look("runner_run_b"), look3: look("runner_jump"), look4: look("runner_crash") }));
  must(world.insert(runner, Run));
  must(world.insert(runner, Hud, { font: font.value as unknown as number, halfWidth, top, bottom: VIEW_BOTTOM }));
  must(world.set(runner, og.Transform, { position: { x: RUNNER_X, y: GROUND_TOP + RUNNER_SIZE.y / 2, z: 1 }, scale: { x: RUNNER_SIZE.x, y: RUNNER_SIZE.y, z: 1 } }));
}

const GROUND_TOP = 0;

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
      must(world.set(e, Actor, { active: true }));
      must(world.set(e, og.Mesh, { visible: true }));
      must(world.set(e, og.Transform, { position: { x, y, z: kind === Kind.Coin ? 0.5 : 0 } }));
      return;
    }
  }
  throw new Error(`every ${args[0]} is in play; the pool holds ${POOLS[args[0] as keyof typeof POOLS]}`);
}

export default og.game({
  tick: 60,
  seed: 1,
  scenes: { main: "scenes/main.ogt" },
  systems: [Director, Jump, Scroll, Spawn, Collide, Animate, HudText],
  render: { antialias: "msaa4" },
  start,
  commands: {
    place: placeCommand,
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
    };
  },
});

export { Phase };
