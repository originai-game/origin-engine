import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "alpha",
  "collision": {
    "against": "depth",
    "bounce": 0,
    "die": true,
    "friction": 0
  },
  "color": [
    0.4000000059604645,
    0.6000000238418579,
    0.11999999731779099,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.10000000149011612,
      [
        1,
        1,
        1,
        0.800000011920929
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0.800000011920929
      ]
    ]
  ],
  "duration": 2,
  "gravity": [
    0,
    -6,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.2000000476837158
  ],
  "maxParticles": 12,
  "rate": 4,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2199999988079071,
      0.30000001192092896,
      0.18000000715255737
    ]
  },
  "size": [
    0.012000000104308128,
    0.019999999552965164
  ],
  "speed": [
    0,
    0
  ]
});
