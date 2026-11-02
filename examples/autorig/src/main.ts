// Characters rigged by the cook from layered art: assets/chars/knight.ogt
// names knight.psd and the humanoid-side template, and the cook makes its
// bones, slots, regions, leg IK and atlas. The scene shows the rig in its
// setup pose (which draws the art exactly as it was drawn) and, beside
// it, the same rig with its feet moved by og.IkTarget2D: a stride and a
// raised knee, the knees bending forwards.

import { og } from "@origingame/engine";
import type { Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function knight(world: World, x: number, stride: number, lift: number): void {
  const rig = must(og.load.rig2d(world, "chars/knight.ogt"));
  const e = must(world.spawn({ name: `knight-${x}`, position: { x, y: 0, z: 0 } }));
  must(world.insert(e, og.Skeleton2D, { rig: rig.rig, layer: 1 }));
  if (stride === 0) return;
  // The feet: the far one back on the ground, the near one forward and lifted.
  const feet: [string, number, number][] = [["ik-foot-l", -stride, 0.12], ["ik-foot-r", stride, lift]];
  for (const [name, dx, y] of feet) {
    const constraint = rig.constraints.indexOf(name);
    const foot = must(world.spawn({ name: `${name}-${x}` }));
    must(world.insert(foot, og.IkTarget2D, { skeleton: e, constraint, target: { x: x + dx, y }, weight: 1 }));
  }
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [],
  start(world) {
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = 2.8;
    camera.position = { x: 0, y: 1.1, z: 50 };
    camera.background = { r: 0.62, g: 0.66, b: 0.72 };
    knight(world, -1.4, 0, 0);
    knight(world, 0, 0.25, 0.25);
    knight(world, 1.4, 0.3, 0.55);
  },
});
