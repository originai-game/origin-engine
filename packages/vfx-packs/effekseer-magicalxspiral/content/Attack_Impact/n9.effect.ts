import { file } from '@origingame/engine/author';
// Effekseer node 9 "ShockWave1"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave4"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        8.0,
        1.0
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
        0.8,
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
