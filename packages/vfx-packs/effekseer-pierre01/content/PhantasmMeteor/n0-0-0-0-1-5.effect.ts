import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.5 "Wind"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre01/textures/Fire_Single"
    },
    "channel": "rgba",
    "uv": {}
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.0625,
        0.3125
      ],
      [
        0.125,
        0.5833
      ],
      [
        0.1875,
        0.8125
      ],
      [
        0.25,
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
