import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "Light Line Render"
export default file.particles({
  "maxParticles": 1875,
  "shape": {
    "kind": "spline",
    "points": [
      [
        0.0,
        0.01,
        0.0
      ],
      [
        -0.0029,
        0.0096,
        0.0
      ],
      [
        -0.0056,
        0.0083,
        0.0
      ],
      [
        -0.0077,
        0.0063,
        0.0
      ],
      [
        -0.0092,
        0.0038,
        0.0
      ],
      [
        -0.01,
        0.001,
        0.0
      ],
      [
        -0.0098,
        -0.002,
        0.0
      ],
      [
        -0.0088,
        -0.0047,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1500
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
    0.05,
    0.1
  ],
  "heightOverLife": [
    [
      0.0,
      20.0
    ],
    [
      1.0,
      20.0
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
        0.1882
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.1882
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
