import { file } from '@origingame/engine/author';
// Effekseer node 1 "Fire"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.001,
      0.5
    ]
  },
  "spread": 0.0,
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
    0.25,
    0.4167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.0,
    4.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.25
    ],
    [
      0.125,
      0.3438
    ],
    [
      0.25,
      0.4375
    ],
    [
      0.375,
      0.5312
    ],
    [
      0.5,
      0.625
    ],
    [
      0.625,
      0.7188
    ],
    [
      0.75,
      0.8125
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.25
    ],
    [
      0.125,
      0.4625
    ],
    [
      0.25,
      0.8
    ],
    [
      0.375,
      1.2625
    ],
    [
      0.5,
      1.85
    ],
    [
      0.625,
      2.5625
    ],
    [
      0.75,
      3.4
    ],
    [
      1.0,
      5.45
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.4844,
      [
        1.0,
        1.0,
        1.0,
        0.5751
      ]
    ],
    [
      0.6719,
      [
        1.0,
        1.0,
        1.0,
        0.2901
      ]
    ],
    [
      0.8438,
      [
        1.0,
        1.0,
        1.0,
        0.098
      ]
    ],
    [
      0.9219,
      [
        1.0,
        1.0,
        1.0,
        0.0388
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
  "rotation": [
    -30.0,
    30.0
  ],
  "angularVelocity": [
    -30.0,
    30.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/fire_tex"
  },
  "blend": "additive"
});
