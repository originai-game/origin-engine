import { file } from '@origingame/engine/author';
// Effekseer node 4 "UVスクロール2"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-effekseer01/textures/Flame01"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        0.9844
      ],
      "offset": [
        0.0,
        0.0156
      ],
      "scroll": [
        5.625,
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
