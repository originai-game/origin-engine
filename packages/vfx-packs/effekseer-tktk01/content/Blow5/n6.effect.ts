import { file } from '@origingame/engine/author';
// Effekseer node 6 "広がりリング外"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Aurora"
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
        0.75,
        1.0
      ],
      [
        0.7812,
        0.6135
      ],
      [
        0.7969,
        0.4743
      ],
      [
        0.8281,
        0.284
      ],
      [
        0.8594,
        0.1843
      ],
      [
        0.9531,
        0.1095
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
