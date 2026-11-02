// Characters rigged by the cook from layered art: each `assets/chars/*.ogt`
// names its PSD and a template (humanoid-side, humanoid-front, quadruped,
// creature), and the cook makes its bones, meshes, IK, atlas and clip
// library. The scene is a parade of one character's library: one copy
// per clip, two rows on a ground line, each clip looping so a film shows
// every one of them over and over: four to a row, three rows, the cells
// spaced (and the camera framed) by the character's size so none reaches
// into its neighbour's, even lying dead. The command `show:<name>`
// (knight, mage, fox, jelly) picks the character, as in
// `og render --commands "0:show:fox"`.

import { og } from "@origingame/engine";
import type { Entity, Result, Rig2DInfo, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/**
 * Each character's rig, its library's clips (left to right, top row
 * first) and its size against a humanoid two units tall.
 */
const CHARACTERS: Record<string, { rig: string; size: number; clips: string[] }> = {
  knight: { rig: "chars/knight.ogt", size: 1, clips: ["idle", "walk", "run", "jump", "attack", "hit", "die", "wave", "cheer", "nod"] },
  mage: { rig: "chars/mage.ogt", size: 1, clips: ["idle", "walk", "run", "jump", "attack", "hit", "die", "wave", "cheer", "shrug"] },
  fox: { rig: "chars/fox.ogt", size: 0.75, clips: ["idle", "walk", "run", "jump", "attack", "hit", "die", "howl", "play", "sniff"] },
  jelly: { rig: "chars/jelly.ogt", size: 0.8, clips: ["idle", "walk", "run", "jump", "attack", "hit", "die", "happy", "wiggle", "look"] },
};
const COLUMNS = 4;
const ROWS = 3;
/** A humanoid's cell: wide enough for it lying dead, tall enough for a jump. */
const SPACING = 4.0;
const ROW = 3.0;

function ground(world: World, y: number, width: number): Entity {
  const e = must(world.spawn({ name: `ground-${y}`, position: { x: 0, y, z: 0 } }));
  must(world.insert(e, og.Sprite, { width, height: 0.04, anchor: { x: 0.5, y: 1 }, color: { r: 0.32, g: 0.35, b: 0.4 }, layer: 0 }));
  return e;
}

function parade(world: World, rig: Rig2DInfo, clips: string[], size: number): Entity[] {
  const [spacing, row] = [SPACING * size, ROW * size];
  const out: Entity[] = [];
  for (let r = 0; r < ROWS; r++) out.push(ground(world, r * row, spacing * COLUMNS));
  clips.forEach((name, i) => {
    const clip = og.anim.clip(rig, name);
    if (clip < 0) throw new Error(`the rig has no clip ${name}`);
    const x = (i % COLUMNS - (COLUMNS - 1) / 2) * spacing;
    const y = (ROWS - 1 - Math.floor(i / COLUMNS)) * row;
    const e = must(world.spawn({ name: `parade-${name}`, position: { x, y, z: 0 } }));
    must(world.insert(e, og.Skeleton2D, { rig: rig.rig, clip, loop: true, layer: 1 }));
    out.push(e);
  });
  return out;
}

let shown: Entity[] = [];
const rigs = new Map<string, Rig2DInfo>();

function show(world: World, name: string): void {
  const c = CHARACTERS[name];
  if (c === undefined) throw new Error(`no character ${name}; one of ${Object.keys(CHARACTERS).join(", ")}`);
  for (const e of shown) must(world.despawn(e));
  let rig = rigs.get(name);
  if (rig === undefined) {
    rig = must(og.load.rig2d(world, c.rig));
    rigs.set(name, rig);
  }
  shown = parade(world, rig, c.clips, c.size);
  // Framed on the cells: the rows' grounds from 0 to two rows up, and a
  // character's height over the top one.
  const camera = must(world.resource(og.Camera));
  camera.height = (SPACING * COLUMNS * 9) / 16 * c.size;
  camera.position = { x: 0, y: (ROW * ROWS * c.size) / 2 - 0.3 * c.size, z: 50 };
}

export default og.game({
  tick: 60,
  seed: 1,
  // A 2D scene: MSAA keeps fast thin parts (a blade's tip, a foot's
  // outline) sharp, where TAA's history leaves ghosts of them behind.
  render: { antialias: "msaa4" },
  systems: [],
  commands: { show: (world, args) => show(world, args[0] ?? "knight") },
  start(world) {
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.background = { r: 0.66, g: 0.7, b: 0.76 };
    show(world, "knight");
  },
});
