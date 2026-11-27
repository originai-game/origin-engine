import { file } from '@origingame/engine/author';
// Effekseer node 8.0 "Fire"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave5"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        0.9375
      ]
    }
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        1.0
      ],
      [
        0.5714,
        1.0
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});
