// A crowd for the lanes: 160 skinned walkers under a camera, so every
// tick's animation evaluation has enough skeletons (more than two of
// og-anim's 64-skeleton grains) to split across the lanes pool when the
// page boots the lanes variant (docs/web-export.md §Lanes).
// tests/web/lane-panic.ts injects a panic into one of those lanes and
// checks that the page reports it. (The fixture was 2,304 boxes for the
// frustum cull until the cull stopped splitting below 131,072 bounds,
// tests/evidence/debt3/cull-lane-grain/NOTES.md.)
import { og, type World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

export const WALKERS = 160;

function start(world: World): void {
  const camera = must(world.resource(og.Camera));
  camera.position = { x: 0, y: 6, z: 14 };
  camera.quaternion = og.transform.lookAt({ x: 0, y: 6, z: 14 }, { x: 0, y: 1, z: 0 });
  const model = must(og.load.gltf(world, "models/walker.glb"));
  const anim = must(og.load.animation(world, "models/walker.glb"));
  const clips = ["walk", "run", "idle", "wave"].map((n) => anim.clips.find((c) => c.name === n)!.clip);
  for (let i = 0; i < WALKERS; i++) {
    const position = { x: ((i % 20) - 9.5) * 0.9, y: 0, z: -Math.floor(i / 20) * 0.9 };
    const e = must(world.instantiate(model, { name: `walker-${i}`, position }));
    world.insert(e, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" });
    world.insert(e, og.AnimationPlayer, { clip: clips[i % clips.length]!, speed: 0.7 + (i % 9) * 0.05, time: (i * 0.071) % 1, loop: true, fade: 0 });
  }
}

export default og.game({ tick: 60, start });
