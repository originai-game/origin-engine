import { file } from '@origingame/engine/author';
// Effekseer node 7 "Wave2"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave5"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        0.5
      ],
      "scroll": [
        0.0,
        0.4688
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
        0.7333,
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
