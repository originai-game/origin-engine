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
        0.07810000330209732,
        0.29409998655319214
      ],
      [
        0.17190000414848328,
        0.5986999869346619
      ],
      [
        0.33329999446868896,
        1
      ],
      [
        0.5,
        0.6251999735832214
      ],
      [
        0.6718999743461609,
        0.32589998841285706
      ],
      [
        0.8281000256538391,
        0.13050000369548798
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
    "texture": ref("base/vfx/effekseer-starter/textures/Particle02"),
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
