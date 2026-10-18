// The game entry: its default export is og.game(...). `og run .` runs it
// headless with the probe; `og dev .` reloads it on every save; `og build .`
// bundles it for the play page.
//
// This is the hello of docs/design.md §3.2 (Health, Velocity, Move) grown
// into a scene: scenes/main.ogt (cooked from assets/scenes/main.ogt) spawns
// the ground, the dropship and the trickster before the first tick; `start`
// gives the ground its mesh and collider, drops a crate above it, puts the
// camera and the UI font in place. The Start button of the menu panel (or S
// or Enter) starts the trickster spinning and lets the crate fall.

import { og, defineComponent, defineResource, defineSystem, bool, f32, Vector3, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";

export const Health = defineComponent("Health", {
  hp: f32(100, { min: 0, max: 1000, tunable: true, label: "Health" }),
  max: f32(100),
});

export const Velocity = defineComponent("Velocity", { v: Vector3() });

/** Turns an entity about +Y; 0 until the player starts. */
export const Spin = defineComponent("Spin", {
  speed: f32(0, { min: -10, max: 10, step: 0.1, unit: "rad", tunable: true, label: "Spin speed" }),
  target: f32(1.5, { min: -10, max: 10, tunable: true, label: "Spin when started" }),
});

/** Whether the player has pressed Start. */
export const Run = defineResource("Run", { started: bool(), chimed: bool() });

export const Move = defineSystem("Move", { reads: [Velocity], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position;
    const v = a.column(Velocity).v;
    for (let i = 0; i < a.count; i++) {
      p[i * 3] += v[i * 3] * time.dt;
      p[i * 3 + 1] += v[i * 3 + 1] * time.dt;
      p[i * 3 + 2] += v[i * 3 + 2] * time.dt;
    }
  }
});

/**
 * The menu panel: the title, the trickster's state and the Start button.
 * A click on Start (or S or Enter) starts the run; the button then reads
 * "Started" and stays inert.
 */
export const Menu = defineSystem("Menu", { reads: [Spin, og.Input], writes: [Run] }, (q, _time, world) => {
  const run = world.resource(Run);
  const input = world.resource(og.Input);
  if (!run.ok || !input.ok) return;
  let speed = 0;
  for (const a of q.archetypes()) if (a.count > 0) speed = a.column(Spin).speed[0]!;
  const started = run.value.started;
  og.ui.column("Hud", { padding: { left: 24, top: 24, right: 24, bottom: 24 } }, () => og.ui.panel("Menu", { padding: { left: 20, top: 16, right: 20, bottom: 16 } }, () => {
    og.ui.label("Title", "Origin Engine", { step: "displaySm" });
    og.ui.label("Status", started ? `开始游戏  ${speed.toFixed(1)} rad/s` : "Press Start to begin", { color: { token: started ? "accentInk" : "fgMuted" } });
    const start = og.ui.button("Start", started ? "Started" : "Start", { variant: "primary", size: "large", disabled: started });
    if (!started && (start.clicked || input.value.pressed("KeyS") || input.value.pressed("Enter"))) run.value.started = true;
  }));
});

/** Once started, every Spin turns at its target speed. */
export const StartSpin = defineSystem("StartSpin", { reads: [Run], writes: [Spin], after: ["Menu"] }, (q, _time, world) => {
  const run = world.resource(Run);
  if (!run.ok || !run.value.started) return;
  for (const a of q.archetypes()) {
    const s = a.column(Spin);
    for (let k = 0; k < a.count; k++) if (s.speed[k] === 0) s.speed[k] = s.target[k]!;
  }
});

/** Once started, held bodies (gravityScale 0) fall. */
export const Drop = defineSystem("Drop", { reads: [Run], writes: [og.RigidBody], after: ["Menu"] }, (q, _time, world) => {
  const run = world.resource(Run);
  if (!run.ok || !run.value.started) return;
  for (const a of q.archetypes()) {
    const g = a.column(og.RigidBody).gravityScale;
    for (let k = 0; k < a.count; k++) if (g[k] === 0) g[k] = 1;
  }
});

/** Once started, the crate's blip plays once (an AudioSource inserted held, released here). */
export const Chime = defineSystem("Chime", { writes: [Run, og.AudioSource], after: ["Menu"] }, (q, _time, world) => {
  const run = world.resource(Run);
  if (!run.ok || !run.value.started || run.value.chimed) return;
  run.value.chimed = true;
  for (const a of q.archetypes()) a.column(og.AudioSource).playing.fill(1);
});

/** Rotates by Spin.speed about +Y: q' = q * (axis-angle about Y). */
export const Rotate = defineSystem("Rotate", { reads: [Spin], writes: [og.Transform], after: ["StartSpin"] }, (q, time) => {
  for (const a of q.archetypes()) {
    const r = a.column(og.Transform).quaternion;
    const s = a.column(Spin).speed;
    for (let i = 0; i < a.count; i++) {
      const half = (s[i] * time.dt) / 2;
      // Small-angle rotation, renormalised: deterministic arithmetic only.
      const sy = half - (half * half * half) / 6;
      const cw = 1 - (half * half) / 2;
      const x = r[i * 4], y = r[i * 4 + 1], z = r[i * 4 + 2], w = r[i * 4 + 3];
      const nx = x * cw - z * sy, ny = y * cw + w * sy, nz = z * cw + x * sy, nw = w * cw - y * sy;
      const n = Math.sqrt(nx * nx + ny * ny + nz * nz + nw * nw);
      r[i * 4] = nx / n;
      r[i * 4 + 1] = ny / n;
      r[i * 4 + 2] = nz / n;
      r[i * 4 + 3] = nw / n;
    }
  }
});

function spawnWave(world: World, count: number): void {
  const ship = og.load.gltf(world, "models/dropship.glb");
  if (!ship.ok) throw new Error(ship.reason);
  for (let i = 0; i < count; i++) {
    const e = world.instantiate(ship.value, { name: `wave-${world.tick}-${i}`, position: { x: -4 + i * 1.5, y: 4, z: -3 } });
    if (!e.ok) throw new Error(e.reason);
    world.insert(e.value, Velocity, { v: { x: 0, y: -0.5, z: 0 } });
  }
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export default og.game({
  tick: 60,
  seed: 1,
  scenes: { main: "scenes/main.ogt" },
  resources: [Run],
  systems: [Move, Menu, StartSpin, Drop, Chime, Rotate],
  // MSAA keeps world-space sprites and text sharp; TAA would blur them across
  // frames. The UI draws over the finished frame either way.
  render: { antialias: "msaa4" },
  start(world) {
    // The camera looks down -Z at the scene from in front.
    const camera = must(world.resource(og.Camera));
    camera.position = { x: 0.4, y: 1.8, z: 8 };
    camera.quaternion = { x: 0, y: 0, z: 0, w: 1 };
    must(world.resource(og.RenderSettings)).sky = true;

    // The ground: a 30 m slab whose top is y = 0, drawn and collided with.
    const ground = world.find("ground");
    if (ground !== null) {
      const slab = must(og.render.boxGeometry(world, { width: 30, height: 0.2, depth: 30 }));
      const stone = must(og.render.standardMaterial(world, { color: { r: 0.18, g: 0.2, b: 0.24 }, roughness: 0.95 }));
      world.set(ground, og.Transform, { position: { x: 0, y: -0.1, z: 0 } });
      world.insert(ground, og.Mesh, { geometry: slab, material: stone, receiveShadow: true });
      world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 30, y: 0.2, z: 30 } });
    }

    // The crate: a dynamic body held in the air (gravityScale 0) until Start.
    const cube = must(og.render.boxGeometry(world, {}));
    const amber = must(og.render.standardMaterial(world, { color: { r: 0.9, g: 0.55, b: 0.08 }, roughness: 0.6 }));
    const crate = must(world.spawn({ name: "crate", position: { x: -2.4, y: 4, z: 0.5 } }));
    world.insert(crate, og.Mesh, { geometry: cube, material: amber, castShadow: true, receiveShadow: true });
    world.insert(crate, og.RigidBody, { gravityScale: 0 });
    world.insert(crate, og.Collider, { shape: SHAPE_BOX });
    const blip = og.load.sound(world, "audio/blip.wav");
    if (blip.ok) world.insert(crate, og.AudioSource, { sound: blip.value, playing: false, spatial: true });

    const trickster = world.find("trickster");
    if (trickster !== null) {
      world.insert(trickster, Spin);
      world.insert(trickster, Health);
    }
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
  },
  commands: { spawn_wave: (world, args) => spawnWave(world, Number(args[0] ?? 3)) },
  facts: (world) => {
    const run = world.resource(Run);
    const trickster = world.find("trickster");
    const spin = trickster === null ? null : world.get(trickster, Spin);
    const crate = world.find("crate");
    const at = crate === null ? null : world.get(crate, og.Transform);
    return {
      started: run.ok && run.value.started,
      spin: spin?.ok ? spin.value.speed : null,
      crateY: at?.ok ? at.value.position.y : null,
    };
  },
});
