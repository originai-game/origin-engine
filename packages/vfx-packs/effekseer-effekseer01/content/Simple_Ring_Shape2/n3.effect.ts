import { file } from '@origingame/engine/author';
// Effekseer node 3 "UVスクロール1"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-effekseer01/textures/Cloud01"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        1.875
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
        0.1429,
        1.0
      ],
      [
        0.8571,
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
