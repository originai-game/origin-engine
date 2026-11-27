import { file } from '@origingame/engine/author';
// Effekseer node 0 "剣跡"
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
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3667,
    0.3667
  ],
  "speed": [
    6.0,
    6.0
  ],
  "size": [
    0.8,
    0.8
  ],
  "heightOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.125,
      0.3223
    ],
    [
      0.25,
      1.1172
    ],
    [
      0.375,
      2.3848
    ],
    [
      0.5,
      4.125
    ],
    [
      0.625,
      6.3379
    ],
    [
      0.75,
      9.0234
    ],
    [
      1.0,
      15.8125
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      -1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1905,
        0.0415,
        1.0
      ]
    ],
    [
      0.5938,
      [
        1.0,
        0.2289,
        0.0745,
        1.0
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.2955,
        0.1462,
        1.0
      ]
    ],
    [
      0.8636,
      [
        1.0,
        0.3885,
        0.2689,
        1.0
      ]
    ],
    [
      0.9531,
      [
        1.0,
        0.5044,
        0.4475,
        0.5132
      ]
    ],
    [
      0.9688,
      [
        1.0,
        0.5297,
        0.4896,
        0.385
      ]
    ],
    [
      0.9844,
      [
        1.0,
        0.5569,
        0.5356,
        0.2174
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.586,
        0.586,
        0.0
      ]
    ]
  ],
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Burst01"
  },
  "blend": "additive"
});
