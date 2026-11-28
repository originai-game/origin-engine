import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        1
      ],
      [
        0.25,
        0.699999988079071
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "additive",
  "main": {
    "texture": ref("base/vfx/cinematic-realism/textures/streak"),
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
