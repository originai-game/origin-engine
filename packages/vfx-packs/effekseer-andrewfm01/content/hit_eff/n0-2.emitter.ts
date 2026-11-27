import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "Motion Sparks"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "spline",
    "points": [
      [
        1.299,
        -0.75,
        0.0
      ],
      [
        1.373,
        -0.6041,
        0.0
      ],
      [
        1.4306,
        -0.4511,
        0.0
      ],
      [
        1.4712,
        -0.2926,
        0.0
      ],
      [
        1.4943,
        -0.1307,
        0.0
      ],
      [
        1.4996,
        0.0327,
        0.0
      ],
      [
        1.4872,
        0.1958,
        0.0
      ],
      [
        1.457,
        0.3565,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 200.0,
  "duration": 0.15,
  "looping": false,
  "lifetime": [
    0.25,
    0.35
  ],
  "speed": [
    24.0,
    24.0
  ],
  "size": [
    0.05,
    0.15
  ],
  "heightOverLife": [
    [
      0.0,
      15.5
    ],
    [
      1.0,
      15.5
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
        0.0478,
        0.0,
        0.251
      ]
    ],
    [
      0.4444,
      [
        1.0,
        0.0478,
        0.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3537,
        0.2158,
        0.7529
      ]
    ],
    [
      0.4444,
      [
        1.0,
        0.3537,
        0.2158,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3537,
        0.2158,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
