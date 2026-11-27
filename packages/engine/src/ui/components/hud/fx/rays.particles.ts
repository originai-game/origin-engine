import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    20,
    34
  ],
  "bursts": [
    {
      "count": 6,
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
        0
      ]
    ],
    [
      0.20000000298023224,
      [
        1,
        1,
        1,
        0.8500000238418579
      ]
    ],
    [
      0.699999988079071,
      [
        1,
        1,
        1,
        0.550000011920929
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
    0,
    0
  ],
  "heightOverLife": [
    [
      0,
      3
    ],
    [
      0.25,
      12
    ],
    [
      1,
      12
    ]
  ],
  "lifetime": [
    2,
    2.4000000953674316
  ],
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "rotation": [
    0,
    180
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    30,
    40
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("@styles/shared/fx/ray.png")
});
