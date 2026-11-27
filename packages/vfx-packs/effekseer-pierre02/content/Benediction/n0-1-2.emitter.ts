import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2 "Shockwave"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.0417
    }
  ],
  "duration": 0.0583,
  "looping": false,
  "lifetime": [
    1.6,
    1.6
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    11.8125,
    15.1875
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8
    ],
    [
      0.3281,
      0.8995
    ],
    [
      1.0,
      1.0
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
      0.0625,
      [
        1.0,
        0.4647,
        0.0356,
        0.1569
      ]
    ],
    [
      0.6875,
      [
        1.0,
        0.4647,
        0.0356,
        0.1569
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Shockwave"
  },
  "blend": "additive"
});
