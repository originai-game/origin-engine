import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0 "Wind"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Single"
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
        0.3553
      ],
      [
        0.1094,
        0.5906
      ],
      [
        0.1562,
        0.7993
      ],
      [
        0.2083,
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
