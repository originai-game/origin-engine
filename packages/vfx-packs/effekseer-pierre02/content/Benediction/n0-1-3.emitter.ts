import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3 "Ray"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 3,
      "interval": 0.0417
    }
  ],
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    1.8,
    1.8
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    39.3437,
    39.3437
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8235
    ],
    [
      0.25,
      0.9199
    ],
    [
      0.5,
      0.9787
    ],
    [
      0.75,
      1.0
    ],
    [
      1.0,
      0.9838
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.1373
    ],
    [
      0.25,
      0.1873
    ],
    [
      0.5,
      0.1998
    ],
    [
      0.75,
      0.1748
    ],
    [
      1.0,
      0.1122
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
      0.1111,
      [
        1.0,
        0.4647,
        0.0356,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.4647,
        0.0356,
        1.0
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
