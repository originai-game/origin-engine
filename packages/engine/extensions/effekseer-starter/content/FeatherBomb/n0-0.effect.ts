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
        0.0625,
        0.5932999849319458
      ],
      [
        0.11110000312328339,
        1
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
    "texture": ref("base/vfx/effekseer-starter/textures/Fire_Single"),
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
