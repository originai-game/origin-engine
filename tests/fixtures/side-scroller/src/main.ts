// A night cave (README.md): a hooded knight, a flipbook driven by a sprite
// animation graph, runs at a slime and slashes; the slash's hit frame blows
// up the rock mound in front of it (og.PixelTerrain), throwing pieces. Torches
// and a moon shaft light it (og.Light2D with soft shadows), og.Follow2D
// follows the knight and the cave's far walls scroll by parallax.
//
//   og run tests/fixtures/side-scroller --size 960x540 --commands "0.3:run;1.5:attack;3:quit"
import { og, defineResource, defineSystem, u32, f32 } from "@origingame/engine";
import type { FlipbookInfo, World } from "@origingame/engine";
import { PROPS } from "./art.ts";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** World units per screen pixel at 960x540: 32 pixels a unit. */
const PX = 1 / 32;
/** The cave's cells: 2 screen pixels each. */
const CELL = 2 * PX;
/** The terrain: its bottom-left corner and its cells. */
const CAVE = { x: -20, y: -11, w: 720, h: 330 };
/** The floor the knight runs on (world y), and where the rock mound stands. */
const FLOOR = -4.5;
const MOUND = { x: 6.2, y: FLOOR + 0.55 };
const RUN_SPEED = 4.2;

/** The knight's states by the sheet's clip names (a GraphDesc.sprite graph). */
const KNIGHT = `type animgraph
version 1
---
param speed
param attack
clip idle "idle"
clip run "run"
clip attack "attack" {
  loop false
}
layer base {
  state idle idle
  state run run
  state attack attack
  entry idle
  transition any attack {
    when attack gt 0.5
  }
  transition idle run {
    when speed gt 0.1
  }
  transition run idle {
    when speed le 0.1
  }
  transition attack run {
    exit 1
    when speed gt 0.1
  }
  transition attack idle {
    exit 1
    when speed le 0.1
  }
}
`;

/** The knight's graph parameters and what the run has seen. */
const Play = defineResource("Play", {
  speed: f32(),
  attack: f32(),
  slashes: u32(),
  detached: u32(),
  steps: u32(),
  hits: u32(),
});

/**
 * Where a vfx2d effect goes once `type vfx2d` lands (og.vfx.spawn of a
 * cooked `fx/<name>.ogt`): `slash` on the attack's hit frame, `dust` on each
 * footstep. Until then it marks the spot with nothing drawn.
 */
function vfx2d(_world: World, _name: "slash" | "dust", _at: { x: number; y: number }): void {
  // og.vfx.spawn(world, { asset: must(og.load.vfx(world, `fx/${name}.ogt`)), position: { ...at, z: 0 } });
}

/** A prop of the packed sheet (props/props.ogt), standing on its feet at `at`, one texel two screen pixels. */
function prop(world: World, sheet: FlipbookInfo, name: keyof typeof PROPS, at: { x: number; y: number }, layer: number, extra: { blend?: "alpha" | "additive"; glow?: number; flipX?: boolean; color?: { r: number; g: number; b: number } } = {}): void {
  const e = must(world.spawn({ name: `${name}@${at.x}`, position: { x: at.x, y: at.y, z: 0 } }));
  must(world.insert(e, og.Sprite, { width: PROPS[name].w * CELL, height: PROPS[name].h * CELL, layer, ...extra }));
  must(world.insert(e, og.Flipbook, { sheet: sheet.sheet, clip: og.anim.clip(sheet, name), playing: false }));
}

function build(world: World): void {
  // The knight: one sheet of three clips, its states named by them.
  const hero = must(og.load.flipbook(world, "hero/hero.ogt"));
  const graph = must(og.anim.graph(world, { text: KNIGHT, sprite: true }));
  const knight = must(world.spawn({ name: "knight", position: { x: -6, y: FLOOR, z: 0 } }));
  must(world.insert(knight, og.Sprite, { width: hero.frameWidth * PX, height: hero.frameHeight * PX, layer: 10 }));
  must(world.insert(knight, og.Flipbook, { sheet: hero.sheet, clip: og.anim.clip(hero, "idle") }));
  must(world.insert(knight, og.AnimationGraph, { asset: graph }));
  must(world.insert(knight, og.Follow2D, { offset: { x: 2.5, y: 3.2 }, smoothTime: 0.25, deadZone: { x: 0.5, y: 4 }, clamp: true, boundsMin: { x: -2, y: -1 }, boundsMax: { x: 20, y: -1 } }));

  const slimeSheet = must(og.load.flipbook(world, "hero/slime.ogt"));
  const slime = must(world.spawn({ name: "slime", position: { x: 9.4, y: FLOOR, z: 0 } }));
  must(world.insert(slime, og.Sprite, { width: slimeSheet.frameWidth * PX, height: slimeSheet.frameHeight * PX, layer: 9, flipX: true }));
  must(world.insert(slime, og.Flipbook, { sheet: slimeSheet.sheet, clip: 0 }));

  // The cave: bedrock walls and floor, rock above them, a mound to blow up.
  const cave = must(world.spawn({ name: "cave", position: { x: CAVE.x, y: CAVE.y, z: 0 } }));
  must(world.insert(cave, og.PixelTerrain, {
    width: CAVE.w, height: CAVE.h, cellSize: CELL, chunk: 32, ground: 0, bedrock: 0, seed: 7,
    color: { r: 0.13, g: 0.14, b: 0.2 }, bedrockColor: { r: 0.045, g: 0.05, b: 0.075 }, grain: 0.22, layer: 5,
    minPiece: 6, maxPieces: 96,
  }));
  const fill = (x: number, y: number, r: number, bedrock = false) => must(og.destruction.fill2d(world, cave, { point: { x, y, z: 0 }, radius: r, bedrock }));
  for (let x = CAVE.x; x < CAVE.x + CAVE.w * CELL; x += 0.5) {
    // A floor of rock over bedrock, uneven by a few cells.
    fill(x, FLOOR - 2 + Math.sin(x * 1.3) * 0.06 + Math.sin(x * 3.1) * 0.04, 2);
    fill(x, FLOOR - 3.4, 2.6, true);
    // The roof, open over the moon.
    if (x < 0.8 || x > 4.4) fill(x, 9.4 + Math.sin(x * 0.7) * 0.5, 2.4, true);
  }
  fill(MOUND.x, MOUND.y - 0.2, 1.1);
  fill(MOUND.x + 0.8, MOUND.y - 0.5, 0.8);

  // The far walls, scrolling slower than the cave (a tilemap of two parallax layers).
  const walls = must(og.load.tilemap(world, "bg/cave.ogt"));
  const back = must(world.spawn({ name: "walls", position: { x: -16, y: 6.2, z: 0 } }));
  must(world.insert(back, og.Tilemap, { map: walls.map, layer: 0, collision: false }));

  // Props from the loose PNGs the cook packed into one sheet.
  const props = must(og.load.flipbook(world, "props/props.ogt"));
  prop(world, props, "moon", { x: 2.4, y: 5.2 }, -15, { glow: 0.15, color: { r: 0.55, g: 0.6, b: 0.75 } });
  prop(world, props, "pillar", { x: -8.5, y: FLOOR }, 4);
  prop(world, props, "pillar", { x: 14.5, y: FLOOR }, 4);
  prop(world, props, "chain", { x: -1.2, y: 4.1 }, 4);
  prop(world, props, "chain", { x: 9.8, y: 3.4 }, 4);
  prop(world, props, "sword", { x: 0.6, y: FLOOR - 0.15 }, 6);
  for (const [x, k] of [[-10, "a"], [-5.5, "b"], [-1, "a"], [3, "b"], [8, "a"], [12.5, "b"]] as const) prop(world, props, `tufts_${k}`, { x, y: FLOOR - 0.06 }, 11);
  prop(world, props, "mushroom", { x: -4.2, y: FLOOR - 0.06 }, 11, { glow: 0.6 });
  prop(world, props, "mushroom", { x: 11.2, y: FLOOR - 0.06 }, 11, { glow: 0.6, flipX: true });
  for (const x of [-3, 8.4]) {
    prop(world, props, "torch", { x, y: FLOOR + 1.6 }, 4);
    prop(world, props, "flame", { x, y: FLOOR + 2.15 }, 120, { blend: "additive", glow: 1.5 });
  }

  // Light: a dark blue cave, two torches and a moon shaft from a hole above.
  const l = must(world.resource(og.Lighting2D));
  l.enabled = true;
  l.ambient = { r: 0.05, g: 0.06, b: 0.1 };
  const torch = (x: number, y: number) => {
    const e = must(world.spawn({ name: `torch${x}`, position: { x, y, z: 0 } }));
    must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.55, b: 0.22 }, intensity: 1.8, radius: 7, falloff: 1.6, sourceRadius: 0.3, castShadows: true }));
    return e;
  };
  torch(-3, FLOOR + 2.3);
  torch(8.4, FLOOR + 2.3);
  const moon = must(world.spawn({ name: "moon", position: { x: 2, y: 8, z: 0 } }));
  must(world.insert(moon, og.Light2D, { kind: "spot", color: { r: 0.5, g: 0.62, b: 1 }, intensity: 1.2, radius: 16, falloff: 1, sourceRadius: 1.2, innerAngle: 0.2, outerAngle: 0.45, castShadows: true }));
  must(world.set(moon, og.Transform, { quaternion: og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, -Math.PI / 2 - 0.25) }));

  const post = must(world.resource(og.PostEffects));
  post.glow = true;
  post.vignette = true;
  post.vignetteIntensity = 0.45;
  const cam = must(world.resource(og.Camera));
  cam.orthographic = true;
  cam.height = 540 * PX;
  cam.position = { x: -3.5, y: -1, z: 10 };
  cam.background = { r: 0.01, g: 0.012, b: 0.025 };
}

/** Moves the knight by the graph's speed and counts what the run reaches. */
const Drive = defineSystem("Drive", { reads: [og.FlipbookEvent, og.PixelDetached], writes: [Play, og.AnimationGraph, og.Transform] }, (_q, time, world) => {
  const play = must(world.resource(Play));
  const knight = world.find("knight");
  if (knight === null) return;
  const graph = must(world.get(knight, og.AnimationGraph));
  const t = must(world.get(knight, og.Transform));
  if (graph.state === 2) og.assert.reachable("the knight slashes");
  const moving = graph.state === 1 ? play.speed : 0;
  must(world.set(knight, og.Transform, { position: { x: t.position.x + moving * time.dt, y: t.position.y, z: 0 } }));
  must(world.set(knight, og.AnimationGraph, { p0: play.speed, p1: play.attack }));
  if (graph.state === 2) play.attack = 0;
  for (const e of world.events(og.FlipbookEvent)) {
    if (e.entity !== knight) continue;
    if (e.name === "step") {
      play.steps += 1;
      vfx2d(world, "dust", { x: t.position.x, y: t.position.y });
    }
    if (e.name === "hit") {
      play.hits += 1;
      const tip = { x: t.position.x + 1.6, y: t.position.y + 1.1 };
      vfx2d(world, "slash", tip);
      // The slash carries into the rock: a blast at the mound throws it.
      const cave = world.find("cave");
      if (cave !== null) must(og.destruction.explode2d(world, { point: { x: MOUND.x - 0.4, y: MOUND.y, z: 0 }, radius: 0.9, debris: 0.6, pieceCells: 30, speed: 9 }));
      must(og.camera.shake(world, 0.35));
    }
  }
  for (const d of world.events(og.PixelDetached)) {
    play.detached += 1;
    if (d.piece !== 0) og.assert.reachable("the mound breaks into pieces");
  }
});

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Play],
  systems: [...og.systems.cameras, Drive],
  start: build,
  commands: {
    run: (world) => { must(world.resource(Play)).speed = RUN_SPEED; },
    stop: (world) => { must(world.resource(Play)).speed = 0; },
    attack: (world) => {
      const play = must(world.resource(Play));
      play.attack = 1;
      play.slashes += 1;
      play.speed = 0;
    },
  },
  commandHelp: {
    run: { usage: "", doc: "The knight runs right." },
    stop: { usage: "", doc: "The knight stops." },
    attack: { usage: "", doc: "The knight stops and slashes; its hit frame blows up the mound." },
  },
  facts: (world) => {
    const play = world.resource(Play);
    const knight = world.find("knight");
    const g = knight === null ? null : world.get(knight, og.AnimationGraph);
    return {
      state: g && g.ok ? g.value.state : -1,
      hits: play.ok ? play.value.hits : 0,
      steps: play.ok ? play.value.steps : 0,
      detached: play.ok ? play.value.detached : 0,
    };
  },
});
