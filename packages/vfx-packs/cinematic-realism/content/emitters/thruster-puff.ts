import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "velocity",
    "stretch": 0.029999999329447746
  },
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        0.5,
        0.5299999713897705,
        0.5600000023841858,
        0.5
      ]
    ],
    [
      1,
      [
        0.20000000298023224,
        0.2199999988079071,
        0.25,
        0
      ]
    ]
  ],
  "drag": 6,
  "duration": 0.15000000596046448,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.10000000149011612,
    0.2199999988079071
  ],
  "looping": false,
  "maxParticles": 40,
  "rate": 220,
  "shape": {
    "kind": "cone",
    "angleDegrees": 13,
    "radius": 0.014999999664723873
  },
  "size": [
    0.029999999329447746,
    0.05000000074505806
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      3.5
    ]
  ],
  "space": "local",
  "speed": [
    5,
    10
  ]
});
