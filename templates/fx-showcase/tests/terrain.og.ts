// The terrain station: an og.CharacterController walking the path up the
// knoll of terrain/valley.ogt, held on the height field (it stands where
// og.terrain.sample says the ground is, grounded, higher than it started),
// above a lake whose planar reflection shows the mountain.
import { moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("terrain", 5, ["terrain-hero", "p2"], ({ t, frame, before, field }) => {
  const p = field<number[]>("terrain-hero", "Transform", "position");
  t.ok(p[0]! > 3000 - 40 + 9, `it walked the path east (x ${p[0]!.toFixed(2)}, from ${3000 - 40})`);
  t.ok(field<boolean>("terrain-hero", "CharacterController", "grounded"), "it stands on the terrain");
  const ground = field<number>("p2", "P2", "ground");
  const start = field<number>("p2", "P2", "start");
  // The capsule's centre is 0.75 m (half its height plus its radius) above its feet.
  t.ok(Math.abs(p[1]! - 0.75 - ground) < 0.15, `its feet are on the sampled ground (y ${p[1]!.toFixed(2)}, ground ${ground.toFixed(2)})`);
  t.ok(ground > start + 0.8, `it climbed the knoll (ground ${ground.toFixed(2)}, ${start.toFixed(2)} at the start)`);
  t.ok(field<number>("p2", "P2", "slope") > 3, "on a slope");
  t.ok(moved(before, frame, 460, 330, 1060, 420) > 0.3, "the lake's waves move");
});
