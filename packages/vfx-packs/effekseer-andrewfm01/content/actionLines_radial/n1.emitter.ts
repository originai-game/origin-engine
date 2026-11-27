import { file } from '@origingame/engine/author';
// Effekseer node 1 "Lines"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.7,
        0.0
      ],
      [
        -0.495,
        0.495,
        0.0
      ],
      [
        -0.7,
        0.0,
        0.0
      ],
      [
        -0.495,
        -0.495,
        0.0
      ],
      [
        0.0,
        -0.7,
        0.0
      ],
      [
        0.495,
        -0.495,
        0.0
      ],
      [
        0.7,
        0.0,
        0.0
      ],
      [
        0.495,
        0.495,
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
    0.025,
    0.1
  ],
  "heightOverLife": [
    [
      0.0,
      40.0
    ],
    [
      1.0,
      40.0
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
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/glow"
  },
  "blend": "alpha"
});
