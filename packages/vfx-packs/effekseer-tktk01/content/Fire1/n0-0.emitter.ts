import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "火メイン"
export default file.particles({
  "maxParticles": 4,
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
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    12.5,
    12.5
  ],
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
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
