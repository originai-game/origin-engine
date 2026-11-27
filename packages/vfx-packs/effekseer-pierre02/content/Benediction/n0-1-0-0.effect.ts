import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0 "Wind"
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
        0.0323,
        0.3571
      ],
      [
        0.0625,
        0.6703
      ],
      [
        0.0968,
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
