import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        0
      ],
      [
        0.058800000697374344,
        1
      ],
      [
        0.17649999260902405,
        1
      ],
      [
        0.7188000082969666,
        0.4049000144004822
      ],
      [
        0.875,
        0.19850000739097595
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "alpha",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/Fire_Single-5d682741"),
    "channel": "rgba",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0
      ],
      "tile": [
        1,
        1
      ],
      "rotate": 0
    }
  }
});
