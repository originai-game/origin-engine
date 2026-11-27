import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "ナイフ"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/naihu"
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
        0.25,
        1.0
      ],
      [
        0.3125,
        0.7254
      ],
      [
        0.375,
        0.5171
      ],
      [
        0.5,
        0.2619
      ],
      [
        0.625,
        0.1585
      ],
      [
        0.8906,
        0.0962
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
