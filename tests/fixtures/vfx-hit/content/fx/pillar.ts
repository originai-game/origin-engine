import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "blend": "additive",
  "doubleSided": true,
  "erosion": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.6000000238418579
      ],
      [
        0.15000000596046448,
        0.10000000149011612
      ],
      [
        0.800000011920929,
        0.15000000596046448
      ],
      [
        1,
        1
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
        -0.5
      ],
      "tile": [
        1,
        2
      ],
      "rotate": 0
    }
  },
  "erosionSoftness": 0.30000001192092896,
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
    "invert": true,
    "power": 3,
    "strength": 1
  },
  "intensity": 4,
  "main": {
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
        -0.800000011920929
      ],
      "tile": [
        2,
        1
      ],
      "rotate": 0
    }
  },
  "ramp": [
    [
      0,
      [
        0.019999999552965164,
        0.15000000596046448,
        0.05000000074505806,
        0.20000000298023224
      ]
    ],
    [
      0.30000001192092896,
      [
        0.15000000596046448,
        1.2000000476837158,
        0.4000000059604645,
        0.6000000238418579
      ]
    ],
    [
      0.6499999761581421,
      [
        0.6000000238418579,
        3,
        1.2000000476837158,
        1
      ]
    ],
    [
      1,
      [
        1.5,
        5,
        2.5,
        1
      ]
    ]
  ]
});
