import { file } from '@origingame/engine/author';
// Effekseer node 2 "Energy"
export default file.particles({
  "maxParticles": 52,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        -4.0,
        0.0
      ],
      [
        2.8284,
        -2.8284,
        0.0
      ],
      [
        4.0,
        0.0,
        0.0
      ],
      [
        2.8284,
        2.8284,
        0.0
      ],
      [
        0.0,
        4.0,
        0.0
      ],
      [
        -2.8284,
        2.8284,
        0.0
      ],
      [
        -4.0,
        0.0,
        0.0
      ],
      [
        -2.8284,
        -2.8284,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 70,
      "interval": 0.0167
    }
  ],
  "duration": 1.1667,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.6,
    0.6
  ],
  "size": [
    0.05,
    0.13
  ],
  "heightOverLife": [
    [
      0.0,
      25.0
    ],
    [
      1.0,
      25.0
    ]
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
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.75,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
  "gravity": [
    0.0,
    -18.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/orb_colored"
  },
  "blend": "additive"
});
