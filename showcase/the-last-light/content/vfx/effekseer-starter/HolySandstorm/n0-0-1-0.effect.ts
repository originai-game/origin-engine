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
        0.22220000624656677,
        1
      ],
      [
        0.33329999446868896,
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
