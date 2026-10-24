// The models station: models/clockwork.ogt, a `type model` the cook built
// from og-geom parts, loaded with og.load.gltf as any model. Its ten parts
// are eight meshes (the three bolts are one, drawn instanced); a ray at the
// gear's root ring hits its front face and one through its hole passes
// through (the cooked BVH, where the parts' triangles are); the frame shows
// the brass gear, the blue vase and the ivory horn.
import { mean, stationPlaytest } from "./station.ts";

export default stationPlaytest("models", 1, ["models"], ({ t, frame, field }) => {
  t.equal(field<number>("models", "Models", "geometries"), 8, "eight meshes for ten parts: the three bolts are one");
  t.ok(field<number>("models", "Models", "ringHits") >= 1, "a ray at the gear's root ring hits the model");
  const z = field<number>("models", "Models", "ringZ");
  // The gear's front face: depth 0.16 from z -0.1, its bevel 0.04 beyond.
  t.ok(Math.abs(z - 0.1) < 0.005, `at the gear's front face (z ${z.toFixed(4)})`);
  t.equal(field<number>("models", "Models", "holeHits"), 0, "a ray through the gear's hole passes through");
  let brass = 0;
  for (let y = 120; y < 520; y++) {
    for (let x = 440; x < 840; x++) {
      const i = (y * frame.width + x) * 4;
      const [r, g, b] = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
      if (r > g + 15 && g > b + 15 && r > 90) brass++;
    }
  }
  t.ok(brass > 20000, `the brass gear at the frame's centre (${brass} pixels)`);
  // The hole shows what is behind the gear, not brass.
  const [hr, hg, hb] = mean(frame, 632, 312, 648, 328);
  t.ok(!(hr > hg + 15 && hg > hb + 15), `the gear's hole is open (${hr.toFixed(0)} ${hg.toFixed(0)} ${hb.toFixed(0)})`);
  const [vr, , vb] = mean(frame, 350, 320, 420, 410);
  t.ok(vb > vr + 20, `the glazed vase on the left (${vr.toFixed(0)} … ${vb.toFixed(0)})`);
});
