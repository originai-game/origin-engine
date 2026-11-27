import { file } from '@origingame/engine/author';
// Effekseer node 6.0 "Node"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        -0.1213,
        0.0,
        -0.4851
      ],
      [
        -0.4185,
        -0.0857,
        -0.2598
      ],
      [
        -0.4706,
        -0.1213,
        0.1176
      ],
      [
        -0.247,
        -0.0857,
        0.4262
      ],
      [
        0.1213,
        0.0,
        0.4851
      ],
      [
        0.4185,
        0.0857,
        0.2598
      ],
      [
        0.4706,
        0.1213,
        -0.1176
      ],
      [
        0.247,
        0.0857,
        -0.4262
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 30,
      "interval": 0.0833
    }
  ],
  "duration": 2.4333,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    12.3693,
    12.3693
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.0,
        0.2195,
        0.502
      ]
    ],
    [
      0.8667,
      [
        0.5295,
        0.0,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      0.8667,
      [
        1.0,
        0.2195,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -54.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/particle"
  },
  "blend": "additive"
});
