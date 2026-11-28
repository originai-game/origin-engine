import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.8999999761581421
      ],
      [
        0.5,
        0.5
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true,
  "erosion": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.15000000596046448
      ],
      [
        0.5,
        0.44999998807907104
      ],
      [
        1,
        0.949999988079071
      ]
    ]
  },
  "noise": {
    "channel": "red",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0.4000000059604645
      ],
      "tile": [
        5,
        3
      ],
      "rotate": 0
    }
  },
  "erosionSoftness": 0.15000000596046448,
  "erosionEdge": {
    "width": 0.05000000074505806,
    "color": [
      2,
      3,
      4,
      1
    ]
  },
  "fresnel": {
    "invert": false,
    "power": 2,
    "strength": 1.5
  },
  "softDepth": 0.10000000149011612
});
