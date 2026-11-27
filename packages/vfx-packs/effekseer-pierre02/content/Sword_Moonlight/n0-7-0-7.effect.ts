import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.7 "Fire_Variation1"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.5,
        0.5
      ],
      "offset": [
        0.5,
        0.5
      ]
    }
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.1,
        1.0
      ],
      [
        0.2969,
        0.8232
      ],
      [
        0.5,
        0.6671
      ],
      [
        0.7344,
        0.2731
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
