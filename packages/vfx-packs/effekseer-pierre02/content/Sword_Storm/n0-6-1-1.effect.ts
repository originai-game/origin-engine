import { file } from '@origingame/engine/author';
// Effekseer node 0.6.1.1 "Charge"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/FractalNoise_1"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        1.0,
        0.5
      ],
      "scroll": [
        0.1172,
        -0.7031
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
        0.3056,
        0.8622
      ],
      [
        0.4375,
        1.0
      ],
      [
        0.5625,
        1.0
      ],
      [
        0.7031,
        0.8482
      ],
      [
        0.8333,
        0.5644
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
