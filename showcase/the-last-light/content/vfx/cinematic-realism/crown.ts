import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.5
      ],
      [
        0.5,
        0.3499999940395355
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "alpha",
  "color": [
    0.8500000238418579,
    0.9200000166893005,
    1,
    1
  ],
  "doubleSided": true,
  "erosion": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.30000001192092896
      ],
      [
        0.4000000059604645,
        0.5
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
        0
      ],
      "tile": [
        12,
        1.2000000476837158
      ],
      "rotate": 0
    }
  },
  "erosionSoftness": 0.11999999731779099,
  "erosionEdge": {
    "width": 0,
    "color": [
      1,
      1,
      1,
      1
    ]
  },
  "fresnel": {
    "invert": false,
    "power": 2.5,
    "strength": 1.2000000476837158
  },
  "softDepth": 0.029999999329447746
});
