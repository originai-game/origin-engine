import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 26,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.800000011920929,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 0.10000000149011612,
  "gravity": [
    0,
    -1300,
    0
  ],
  "heightOverLife": [
    [
      0,
      1
    ],
    [
      0.14000000059604645,
      0.20000000298023224
    ],
    [
      0.2800000011920929,
      1
    ],
    [
      0.41999998688697815,
      0.20000000298023224
    ],
    [
      0.5600000023841858,
      1
    ],
    [
      0.699999988079071,
      0.20000000298023224
    ],
    [
      0.8399999737739563,
      1
    ],
    [
      1,
      0.20000000298023224
    ]
  ],
  "lifetime": [
    1.2999999523162842,
    1.7000000476837158
  ],
  "looping": false,
  "maxParticles": 40,
  "rate": 0,
  "rotation": [
    -20,
    20
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 30,
    "radius": 4
  },
  "size": [
    18,
    26
  ],
  "speed": [
    420,
    620
  ],
  "texture": ref("@styles/shared/fx/coin.png")
});
