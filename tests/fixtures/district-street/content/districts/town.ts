import { file, ref } from "@origingame/engine/author";

export default file.district({
  seed: 7,
  region: { kind: "area", points: [[0, 0], [240, 0], [240, 160], [0, 160]] },
  streets: { grid: { size: [60, 80], width: 10, sidewalk: 3 }, kerb: 0.15 },
  lots: { width: [6, 14], depth: [10, 18] },
  buildings: {
    density: 0.9,
    height: [9, 24],
    heightNoise: { frequency: 0.01, share: 0.6 },
    rules: [{ grammar: ref("base/grammars/terrace") }],
  },
  props: [{
    name: "lamp",
    model: ref("base/models/lamp"),
    every: 22,
    jitter: 1,
    kerb: 0.5,
    corners: true,
    collider: { kind: "capsule", radius: 0.12, height: 4 },
  }],
  materials: ref("base/materials/london"),
  terrain: { terrain: ref("base/terrain/land"), depth: 0.25 },
  cells: { size: 64, load: 160 },
});
