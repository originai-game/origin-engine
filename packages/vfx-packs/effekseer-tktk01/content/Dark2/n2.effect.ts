import { file } from '@origingame/engine/author';
// Effekseer node 2 "広がる魔法陣"
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
        0.1562,
        0.9252
      ],
      [
        0.3281,
        0.7803
      ],
      [
        0.4844,
        0.6099
      ],
      [
        0.7344,
        0.3036
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
