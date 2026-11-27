import { file } from '@origingame/engine/author';
// Effekseer node 0 "Lines"
export default file.particles({
  "maxParticles": 2500,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.75,
        0.0
      ],
      [
        -0.5303,
        0.5303,
        0.0
      ],
      [
        -0.75,
        0.0,
        0.0
      ],
      [
        -0.5303,
        -0.5303,
        0.0
      ],
      [
        0.0,
        -0.75,
        0.0
      ],
      [
        0.5303,
        -0.5303,
        0.0
      ],
      [
        0.75,
        0.0,
        0.0
      ],
      [
        0.5303,
        0.5303,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 59999.9961,
  "duration": 0.0333,
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
    0.075,
    0.2
  ],
  "heightOverLife": [
    [
      0.0,
      32.7273
    ],
    [
      1.0,
      32.7273
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
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/glow"
  },
  "blend": "alpha"
});
