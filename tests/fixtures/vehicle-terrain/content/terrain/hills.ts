import { file } from "@origingame/engine/author";

// Low rolling hills, 160 m square, a sample every 1.25 m; no erosion.
export default file.terrain({
  source: { kind: "noise", columns: 129, rows: 129, height: [0,6], noise: { octaves: 3, frequency: 1.5, gain: 0.4 } },
  seed: 7, size: [160,160],
  process: { biomes: [{ layer: 0, slope: [0,12] }, { layer: 1, slope: [12,90] }] },
  layers: [
    { color: [0.3,0.42,0.2], roughness: 0.95 },
    { color: [0.42,0.33,0.22], roughness: 1 },
  ],
});
