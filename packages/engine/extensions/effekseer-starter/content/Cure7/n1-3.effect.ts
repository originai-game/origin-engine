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
        0.7031000256538391,
        0.9663000106811523
      ],
      [
        0.8125,
        0.8395000100135803
      ],
      [
        0.8593999743461609,
        0.7197999954223633
      ],
      [
        0.9061999917030334,
        0.5465999841690063
      ],
      [
        0.9531000256538391,
        0.3098999857902527
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
    "texture": ref("base/vfx/effekseer-starter/textures/Shock_wave_ring0011"),
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
