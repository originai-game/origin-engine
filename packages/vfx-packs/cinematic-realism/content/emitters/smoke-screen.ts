import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -8,
    8
  ],
  "blend": "alpha",
  "color": [
    0.8500000238418579,
    0.8600000143051147,
    0.8799999952316284,
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
      0.07999999821186066,
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
        0.699999988079071
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
  "drag": 1.2000000476837158,
  "duration": 8,
  "gravity": [
    0,
    0.07999999821186066,
    0
  ],
  "lifetime": [
    7,
    10
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 160,
  "noise": {
    "frequency": 0.4000000059604645,
    "scroll": 0.20000000298023224,
    "strength": 0.20000000298023224
  },
  "rate": 14,
  "rateOverTime": [
    [
      0,
      1
    ],
    [
      0.800000011920929,
      1
    ],
    [
      1,
      0.30000001192092896
    ]
  ],
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 70,
    "radius": 0.15000000596046448
  },
  "sheet": {
    "blend": true,
    "columns": 16,
    "frames": 64,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 4
  },
  "size": [
    1.2000000476837158,
    1.7999999523162842
  ],
  "sizeOverLife": [
    [
      0,
      0.4000000059604645
    ],
    [
      0.20000000298023224,
      1
    ],
    [
      1,
      2.5
    ]
  ],
  "softDistance": 0.6000000238418579,
  "speed": [
    0.4000000059604645,
    1.2000000476837158
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
