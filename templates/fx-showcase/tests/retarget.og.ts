// The retarget station: one walk from the animation library on three
// characters. og.anim.humanoid found every bone each rig has (the walker's
// 12 Blender names, the strider's 21 Mixamo names through its rig file, the
// stout's 21 Unreal names), and root motion carried each at its own
// stride: the strider (legs 1.267 times the library's) furthest, the stout
// (0.689 times) least, in the ratio of their legs.
import { stationPlaytest } from "./station.ts";

export default stationPlaytest("retarget", 2.5, ["anim-physics", "rt-walker", "rt-strider", "rt-stout"], ({ t, field }) => {
  t.equal(field<number>("anim-physics", "AnimPhysics", "walkerBones"), 12, "the walker's bones were found");
  t.equal(field<number>("anim-physics", "AnimPhysics", "striderBones"), 21, "the strider's bones were found (its rig file)");
  t.equal(field<number>("anim-physics", "AnimPhysics", "stoutBones"), 21, "the stout's bones were found");
  const start = 1400 - 5;
  const went = (e: string) => field<number[]>(e, "Transform", "position")[0]! - start;
  const [walker, strider, stout] = [went("rt-walker"), went("rt-strider"), went("rt-stout")];
  t.ok(walker > 2, `the walker walked (${walker.toFixed(2)} m)`);
  t.ok(Math.abs(strider / walker - 1.267) < 0.05, `the strider's stride is its legs' (${(strider / walker).toFixed(3)} of the walker's)`);
  t.ok(Math.abs(stout / walker - 0.689) < 0.05, `the stout's stride is its legs' (${(stout / walker).toFixed(3)} of the walker's)`);
  for (const e of ["rt-walker", "rt-strider", "rt-stout"]) {
    const [, y, z] = field<number[]>(e, "Transform", "position");
    t.ok(Math.abs(y!) < 1e-4, `${e} stays on the ground plane (y ${y!.toFixed(5)})`);
    t.ok(Math.abs(z! - { "rt-walker": 0, "rt-strider": -1.4, "rt-stout": 1.4 }[e]!) < 1e-3, `${e} walks straight along x (z ${z!.toFixed(4)})`);
  }
});
