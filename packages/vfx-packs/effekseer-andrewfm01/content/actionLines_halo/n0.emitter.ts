import { file } from '@origingame/engine/author';
// Effekseer node 0 "Lines"
export default file.particles({
  "maxParticles": 438,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        1.25,
        0.0
      ],
      [
        -0.8839,
        0.8839,
        0.0
      ],
      [
        -1.25,
        0.0,
        0.0
      ],
      [
        -0.8839,
        -0.8839,
        0.0
      ],
      [
        0.0,
        -1.25,
        0.0
      ],
      [
        0.8839,
        -0.8839,
        0.0
      ],
      [
        1.25,
        0.0,
        0.0
      ],
      [
        0.8839,
        0.8839,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 350
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
    0.075
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
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/glow"
  },
  "blend": "alpha"
});
