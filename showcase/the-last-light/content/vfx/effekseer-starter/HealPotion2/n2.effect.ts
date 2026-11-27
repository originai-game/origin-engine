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
        0.5,
        1
      ],
      [
        0.5781000256538391,
        0.8999999761581421
      ],
      [
        0.75,
        0.8414999842643738
      ],
      [
        0.8281000256538391,
        0.748199999332428
      ],
      [
        0.875,
        0.6341999769210815
      ],
      [
        0.9218999743461609,
        0.4602000117301941
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true,
  "main": {
    "texture": ref("base/vfx/effekseer-starter/textures/AuroraRing"),
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
