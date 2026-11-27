import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "Ray"
export default file.particles({
  "maxParticles": 75,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.0625,
      1.125,
      5.0625
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 12,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.8333,
    1.8333
  ],
  "speed": [
    0.1169,
    0.4677
  ],
  "size": [
    -1.125,
    3.375
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.66
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      8.0
    ],
    [
      1.0,
      24.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.4647,
        0.0356,
        0.0
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.4647,
        0.0356,
        0.502
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.4647,
        0.0356,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.4647,
        0.0356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
