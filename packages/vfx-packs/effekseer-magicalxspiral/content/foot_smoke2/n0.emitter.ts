import { file } from '@origingame/engine/author';
// Effekseer node 0 "Smoke"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    0.2165,
    0.866
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.05,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.4375,
      [
        1.0,
        1.0,
        1.0,
        0.1798
      ]
    ],
    [
      0.5625,
      [
        1.0,
        1.0,
        1.0,
        0.1222
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/smoke_01"
  },
  "sheet": {
    "columns": 4,
    "rows": 4,
    "frames": 16,
    "mode": {
      "kind": "life",
      "cycles": 1.0
    }
  },
  "blend": "additive"
});
