import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "attractors": [
    {
      "position": [
        0,
        0,
        0
      ],
      "radius": 0,
      "strength": 7
    }
  ],
  "blend": "additive",
  "colorOverLife": [
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
      0.30000001192092896,
      [
        0.6000000238418579,
        1.2000000476837158,
        2.5999999046325684,
        1
      ]
    ],
    [
      1,
      [
        1,
        1.7999999523162842,
        3.200000047683716,
        1
      ]
    ]
  ],
  "drag": 0.5,
  "duration": 2,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.5,
    0.800000011920929
  ],
  "maxParticles": 64,
  "shape": {
    "kind": "sphere",
    "radius": 0.800000011920929,
    "surface": true
  },
  "size": [
    0.012000000104308128,
    0.02500000037252903
  ],
  "space": "local",
  "speed": [
    0,
    0
  ]
});
