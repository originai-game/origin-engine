import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "地面の魔法陣"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/circle4"
    },
    "channel": "rgba",
    "uv": {}
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        1.0
      ],
      [
        0.4219,
        0.9625
      ],
      [
        0.5312,
        0.9156
      ],
      [
        0.625,
        0.8395
      ],
      [
        0.7344,
        0.6949
      ],
      [
        0.8281,
        0.5118
      ],
      [
        0.9219,
        0.2636
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
