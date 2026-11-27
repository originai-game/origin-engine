import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0 "Ring"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-suzuki01/textures/fire"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        1.7578,
        0.0
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
        0.0444,
        1.0
      ],
      [
        0.5556,
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
