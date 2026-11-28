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
      "strength": 5
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
        1,
        0.800000011920929,
        2.5999999046325684,
        1
      ]
    ],
    [
      1,
      [
        0.6000000238418579,
        1.399999976158142,
        3,
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
    0.699999988079071,
    1.100000023841858
  ],
  "maxParticles": 48,
  "rate": 24,
  "shape": {
    "kind": "sphere",
    "radius": 1.100000023841858,
    "surface": true
  },
  "size": [
    0.012000000104308128,
    0.024000000208616257
  ],
  "space": "local",
  "speed": [
    0,
    0
  ]
});
