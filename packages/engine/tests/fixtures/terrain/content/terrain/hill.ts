import { file, ref } from '@origingame/engine/author';

// A 64 m hill: 129 × 129 half-metre samples, intensities spanning 0–10 m.
export default file.terrain({
  source: { kind: 'image', source: 'assets/terrain/hill.png', height: [0, 10] },
  size: [64, 64],
  tile: 64,
  layers: [{ albedo: ref('base/textures/grass'), tile: 4, roughness: 0.9 }],
});
