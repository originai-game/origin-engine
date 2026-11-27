import { file } from "@origingame/engine/author";

export default file.terrain({
  source: { kind: "noise", columns: 257, rows: 257, height: [0,0] },
  seed: 1, size: [512,512], tile: 128,
  layers: [{ color: [0.18,0.22,0.1], roughness: 0.95, tile: 8 }],
});
