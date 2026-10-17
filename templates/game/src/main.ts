// The game entry: its default export is og.game(...). `og run .` runs it
// headless with the probe; `og dev .` reloads it on every save; `og build .`
// bundles it for the play page.
//
// This is the hello of docs/design.md §3.2 (Health, Velocity, Move) grown
// into a scene: scenes/main.ogt (cooked from assets/scenes/main.ogt) spawns
// the ground, the dropship and the trickster before the first tick; `start`
// puts the camera, the light and the label in place; pressing S, Space or
// Enter (or clicking) starts the trickster spinning.

import { og, defineComponent, defineSystem, f32, f64, Vector3 } from "@origingame/engine";
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

/** World-space text above an entity, drawn with a cooked MSDF font. */
export const Label = defineComponent("Label", {
  font: f64(),
  height: f32(2.6, { unit: "m" }),
});

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

/** The first S, Space, Enter or click sets every Spin to its target speed. */
export const StartOnInput = defineSystem("StartOnInput", { reads: [og.Input], writes: [Spin] }, (q, _time, world) => {
  const input = world.resource(og.Input);
  if (!input.ok) return;
  const i = input.value;
  if (!(i.pressed("KeyS") || i.pressed("Space") || i.pressed("Enter") || i.buttonPressed("left"))) return;
  for (const a of q.archetypes()) {
    const s = a.column(Spin);
    for (let k = 0; k < a.count; k++) if (s.speed[k] === 0) s.speed[k] = s.target[k];
  }
});

/** Rotates by Spin.speed about +Y: q' = q * (axis-angle about Y). */
export const Rotate = defineSystem("Rotate", { reads: [Spin], writes: [og.Transform], after: ["StartOnInput"] }, (q, time) => {
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

/** The label above each Spin: a prompt before the start, the speed after. */
export const Hud = defineSystem("Hud", { reads: [Label, Spin, og.Transform], after: ["Rotate"] }, (q, _time, world) => {
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position;
    const l = a.column(Label);
    const s = a.column(Spin).speed;
    for (let i = 0; i < a.count; i++) {
      const started = s[i] !== 0;
      og.render.text(world, {
        font: l.font[i] as never,
        text: started ? `开始游戏  ${s[i].toFixed(1)} rad/s` : "Press S to start",
        position: { x: p[i * 3] - 1.6, y: p[i * 3 + 1] + l.height[i], z: p[i * 3 + 2] },
        size: 0.32,
        color: started ? { r: 1, g: 0.85, b: 0.3 } : { r: 1, g: 1, b: 1 },
      });
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

export default og.game({
  tick: 60,
  seed: 1,
  scenes: { main: "scenes/main.ogt" },
  systems: [Move, StartOnInput, Rotate, Hud],
  render: { antialias: "tier" },
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) {
      camera.value.position = { x: 0.6, y: 2.4, z: 7.5 };
      camera.value.lookAt({ x: 1, y: 1.2, z: 0 });
    }
    const settings = world.resource(og.RenderSettings);
    if (settings.ok) settings.value.sky = true;
    const trickster = world.find("trickster");
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    if (trickster !== null) {
      world.insert(trickster, Spin);
      world.insert(trickster, Health);
      if (font.ok) world.insert(trickster, Label, { font: font.value as unknown as number });
    }
  },
  commands: { spawn_wave: (world, args) => spawnWave(world, Number(args[0] ?? 3)) },
  facts: (world) => {
    const trickster = world.find("trickster");
    const spin = trickster === null ? null : world.get(trickster, Spin);
    return { started: spin?.ok === true && spin.value.speed !== 0, spin: spin?.ok ? spin.value.speed : null };
  },
});
