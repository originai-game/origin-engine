import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "溶岩本体"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/magma"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.9766,
        0.9766
      ],
      "scroll": [
        0.0,
        0.2344
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
        0.2222,
        1.0
      ],
      [
        0.3333,
        1.0
      ],
      [
        0.4375,
        0.8771
      ],
      [
        0.5625,
        0.7006
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
