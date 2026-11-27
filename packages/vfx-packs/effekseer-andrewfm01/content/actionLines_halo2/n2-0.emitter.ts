import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "Mid Line Render"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "spline",
    "points": [
      [
        0.0,
        0.01,
        0.0
      ],
      [
        -0.001,
        0.0099,
        0.0
      ],
      [
        -0.0021,
        0.0098,
        0.0
      ],
      [
        -0.0031,
        0.0095,
        0.0
      ],
      [
        -0.004,
        0.0092,
        0.0
      ],
      [
        -0.005,
        0.0087,
        0.0
      ],
      [
        -0.0058,
        0.0081,
        0.0
      ],
      [
        -0.0066,
        0.0075,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 400
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
      16.6667
    ],
    [
      1.0,
      16.6667
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
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
