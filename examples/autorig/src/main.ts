// Characters rigged by the cook from layered art: assets/chars/knight.ogt
// names knight.psd and the humanoid-side template, and the cook makes its
// bones, meshes, leg IK, atlas and clip library. The scene is a parade
// of the library: one knight per clip, two rows on a ground line, each
// clip looping so a film shows every one of them over and over.

import { og } from "@origingame/engine";
import type { Result, Rig2DInfo, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The library's clips, left to right, top row first. */
const CLIPS = ["idle", "walk", "run", "jump", "attack", "hit", "die", "wave", "cheer", "nod"];
const COLUMNS = 5;
const SPACING = 1.9;
const ROW = 2.9;

function ground(world: World, y: number): void {
  const e = must(world.spawn({ name: `ground-${y}`, position: { x: 0, y, z: 0 } }));
  must(world.insert(e, og.Sprite, { width: SPACING * COLUMNS + 1, height: 0.05, anchor: { x: 0.5, y: 1 }, color: { r: 0.32, g: 0.35, b: 0.4 }, layer: 0 }));
}

function parade(world: World, rig: Rig2DInfo): void {
  CLIPS.forEach((name, i) => {
    const clip = og.anim.clip(rig, name);
    if (clip < 0) throw new Error(`the rig has no clip ${name}`);
    const x = (i % COLUMNS - (COLUMNS - 1) / 2) * SPACING;
    const y = i < COLUMNS ? ROW : 0;
    const e = must(world.spawn({ name: `knight-${name}`, position: { x, y, z: 0 } }));
    must(world.insert(e, og.Skeleton2D, { rig: rig.rig, clip, loop: true, layer: 1 }));
  });
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [],
  start(world) {
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = 6.0;
    camera.position = { x: 0, y: 2.55, z: 50 };
    camera.background = { r: 0.66, g: 0.7, b: 0.76 };
    ground(world, ROW);
    ground(world, 0);
    parade(world, must(og.load.rig2d(world, "chars/knight.ogt")));
  },
});
