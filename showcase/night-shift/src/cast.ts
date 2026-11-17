// The cast: the courier and his hover-bike in the foreground, and the
// pedestrians walking the alley under their umbrellas. The courier is posed
// on the CPU (one skeleton, cross-fading clips); the pedestrians are a GPU
// crowd, each a few seconds out of step with the others.
import { defineComponent, defineSystem, f32, og } from "@origingame/engine";
import type { Asset, ClipAsset, Entity, SkeletonAsset, World } from "@origingame/engine";
import { NEON } from "./palette.ts";
import { must, v3, yaw } from "./util.ts";

interface Rigged { readonly model: Asset; readonly skeleton: SkeletonAsset; readonly clips: ReadonlyMap<string, ClipAsset> }
function rigged(world: World, path: string): Rigged {
  const anim = must(og.load.animation(world, path));
  return { model: must(og.load.gltf(world, path)), skeleton: anim.skeleton, clips: new Map(anim.clips.map((c) => [c.name, c.clip])) };
}
function clip(r: Rigged, name: string): ClipAsset {
  const c = r.clips.get(name);
  if (c === undefined) throw new Error(`no clip "${name}" (has ${[...r.clips.keys()].join(", ")})`);
  return c;
}

/** One walker of the alley: which model, where it starts, which way it walks (+1 towards the camera) and how far into its stride it is. */
export interface Walker { readonly model: "a" | "b" | "c"; readonly x: number; readonly z: number; readonly towards: 1 | -1; readonly offset: number; readonly stands?: boolean }
const WALKERS: readonly Walker[] = [
  { model: "a", x: -1.3, z: -4.5, towards: -1, offset: 0.0 },
  { model: "b", x: 0.5, z: -8.0, towards: 1, offset: 0.5 },
  { model: "c", x: -0.5, z: -10.5, towards: -1, offset: 0.9 },
  { model: "a", x: 1.7, z: -13.0, towards: 1, offset: 0.3 },
  { model: "b", x: -2.0, z: -15.5, towards: -1, offset: 1.1 },
  { model: "c", x: 2.6, z: -6.2, towards: 1, offset: 0.0, stands: true },
  { model: "a", x: 0.2, z: -18.5, towards: 1, offset: 0.7 },
  { model: "c", x: -0.9, z: -21.5, towards: -1, offset: 0.2 },
];
/** Walking pace, and the stretch of alley the walkers loop over. */
const PACE = 1.25;
const ALLEY_NEAR = -3;
const ALLEY_FAR = -26;

/** The stretch of the courier's idle clip where he stands and looks down the alley (its middle turns him round to look behind): from this second, for this long, across the clip's loop point at 4 s. */
const STAND_FROM = 3.3;
const STAND_FOR = 1.2;
const IDLE_LENGTH = 4;

/** A walker: +1 walks towards the camera, -1 away. */
export const Pedestrian = defineComponent("Pedestrian", { towards: f32() });

export interface Cast { readonly courier: Entity; readonly bike: Entity }

export function buildCast(world: World): Cast {
  // The courier: three-quarter back to the camera, looking down the alley.
  const man = rigged(world, "models/courier.glb");
  const courier = must(world.instantiate(man.model, { name: "courier", position: v3(0.75, 0, 5.0), quaternion: yaw(165) }));
  must(world.insert(courier, og.Skeleton, { asset: man.skeleton, rootMotion: "off" }));
  must(world.insert(courier, og.AnimationPlayer, { clip: clip(man, "Idle"), loop: true, speed: 0, time: STAND_FROM }));
  const packLight = must(world.spawn({ name: "pack-light", parent: courier, position: v3(0, 1.25, -0.55) }));
  must(world.insert(packLight, og.PointLight, { color: NEON.cyan, intensity: 0.8, distance: 4 }));

  // The hover-bike: 3.1 m long, floating a hand above the street, nose towards the alley.
  const hull = must(og.load.gltf(world, "models/bike.glb"));
  const bike = must(world.instantiate(hull, { name: "bike", position: v3(2.7, 0.78, 3.5), quaternion: yaw(28), scale: v3(1.65, 1.65, 1.65) }));
  const under = must(world.spawn({ name: "bike-underglow", parent: bike, position: v3(0, -0.3, 0) }));
  must(world.insert(under, og.PointLight, { color: NEON.cyan, intensity: 4, distance: 4 }));

  // The pedestrians: a crowd on the GPU.
  const peds = { a: rigged(world, "models/ped_a.glb"), b: rigged(world, "models/ped_b.glb"), c: rigged(world, "models/ped_c.glb") } as const;
  for (const [i, w] of WALKERS.entries()) {
    const r = peds[w.model];
    const e = must(world.instantiate(r.model, { name: `pedestrian-${i}`, position: v3(w.x, 0, w.z), quaternion: yaw(w.towards > 0 ? 0 : 180) }));
    const playing = w.stands ? (r.clips.get("Stand_and_Chat") ?? clip(r, "Idle")) : clip(r, "Walk_with_Umbrella_inplace");
    must(world.insert(e, og.CrowdAnimation, { skeleton: r.skeleton, clip: playing, offset: w.offset }));
    if (!w.stands) must(world.insert(e, Pedestrian, { towards: w.towards }));
  }
  return { courier, bike };
}

/** Walks the pedestrians along the alley, wrapping each at its end. */
export const Walk = defineSystem("Walk", { reads: [Pedestrian], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const towards = a.column(Pedestrian).towards;
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) {
      let z = p[i * 3 + 2]! + towards[i]! * PACE * time.dt;
      if (z > ALLEY_NEAR) z = ALLEY_FAR;
      if (z < ALLEY_FAR) z = ALLEY_NEAR;
      p[i * 3 + 2] = z;
    }
  }
});

/** Keeps the courier standing: his idle clip played to and fro over its standing stretch, at a third of its speed. */
export const Stand = defineSystem("Stand", { writes: [og.AnimationPlayer] }, (_q, time, world) => {
  const courier = world.find("courier");
  if (courier === null) return;
  const u = ((time.tick / 60) * 0.35) % (2 * STAND_FOR);
  const along = u < STAND_FOR ? u : 2 * STAND_FOR - u;
  must(world.set(courier, og.AnimationPlayer, { time: (STAND_FROM + along) % IDLE_LENGTH }));
});
