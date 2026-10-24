// The islands station: a terrain the cook generated (terrain/islands.ogt's
// `generate` block, no heightmap) loaded and drawn as any terrain, its
// sea an og.Water at og.terrain.load's seaLevel: the island rises above
// the sea at its centre and lies under it at its corners, and the frame
// shows open sea, a beach and the island's meadows.
import { mean, stationPlaytest } from "./station.ts";

export default stationPlaytest("islands", 2, ["islands", "islands-sea"], ({ t, frame, field }) => {
  const sea = field<number>("islands", "Gen", "seaLevel");
  t.equal(sea, 2, "og.terrain.load answers the generated terrain's sea level (generate { sea 2 })");
  const p = field<number[]>("islands-sea", "Transform", "position");
  t.equal(p[1], sea, "the water stands at it");
  const centre = field<number>("islands", "Gen", "centre");
  const corner = field<number>("islands", "Gen", "corner");
  t.ok(centre > sea + 5, `the island rises above the sea at its centre (${centre.toFixed(2)} m)`);
  t.ok(corner < sea, `and lies under it at its corner (${corner.toFixed(2)} m)`);
  const [r, g, b] = mean(frame, 20, 640, 220, 710);
  t.ok(b > r + 25, `open sea at the frame's foot (${r.toFixed(0)} ${g.toFixed(0)} ${b.toFixed(0)})`);
  let sand = 0;
  let grass = 0;
  for (let y = 200; y < 600; y++) {
    for (let x = 200; x < 1100; x++) {
      const i = (y * frame.width + x) * 4;
      const [pr, pg, pb] = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
      if (pr >= pg && pg >= pb && pr - pb >= 25 && pr > 150) sand++;
      else if (pg > pr + 8 && pg > pb + 8) grass++;
    }
  }
  t.ok(sand > 1000, `a beach between the sea and the land (${sand} sand pixels)`);
  t.ok(grass > 20000, `the biomes' grass on the flats (${grass} pixels)`);
});
