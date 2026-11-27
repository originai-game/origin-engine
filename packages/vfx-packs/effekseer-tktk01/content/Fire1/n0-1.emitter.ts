import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "火メイン"
export default file.particles({
  "maxParticles": 52,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        10.0,
        0.0
      ],
      [
        -7.0711,
        7.0711,
        0.0
      ],
      [
        -10.0,
        0.0,
        0.0
      ],
      [
        -7.0711,
        -7.0711,
        0.0
      ],
      [
        0.0,
        -10.0,
        0.0
      ],
      [
        7.0711,
        -7.0711,
        0.0
      ],
      [
        10.0,
        0.0,
        0.0
      ],
      [
        7.0711,
        7.0711,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 1.4583,
  "looping": false,
  "lifetime": [
    0.0833,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    12.5,
    12.5
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.9323,
        0.3941,
        0.0242,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.9323,
        0.3941,
        0.0242,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -21600.0,
    21600.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
