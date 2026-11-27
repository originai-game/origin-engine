import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "doubleSided": true,
  "erosion": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.05000000074505806
      ],
      [
        1,
        0.8999999761581421
      ]
    ]
  },
  "noise": {
    "texture": ref("base/textures/noise"),
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
        2,
        2
      ],
      "rotate": 0
    }
  },
  "erosionSoftness": 0.25,
  "erosionEdge": {
    "width": 0,
    "color": [
      1,
      1,
      1,
      1
    ]
  },
  "intensity": 1.399999976158142,
  "main": {
    "texture": ref("base/textures/noise"),
    "channel": "red",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": true,
      "scroll": [
        0.10000000149011612,
        -1.2000000476837158
      ],
      "tile": [
        5,
        0.6000000238418579
      ],
      "rotate": 0
    }
  },
  "ramp": [
    [
      0,
      [
        0,
        0,
        0,
        0
      ]
    ],
    [
      0.4000000059604645,
      [
        1.2000000476837158,
        0.3499999940395355,
        0.05000000074505806,
        0.6000000238418579
      ]
    ],
    [
      0.800000011920929,
      [
        4,
        1.7999999523162842,
        0.5,
        1
      ]
    ],
    [
      1,
      [
        6,
        4,
        2.5,
        1
      ]
    ]
  ]
});
