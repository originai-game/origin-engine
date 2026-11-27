import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "地面柱カラー"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/7irohannten"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        0.4961
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
        0.5,
        1.0
      ],
      [
        0.5469,
        0.9568
      ],
      [
        0.6094,
        0.8516
      ],
      [
        0.6719,
        0.7079
      ],
      [
        0.875,
        0.1814
      ],
      [
        0.9375,
        0.0649
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
