import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "Node"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        0.7
      ],
      [
        -0.495,
        0.0,
        0.495
      ],
      [
        -0.7,
        0.0,
        0.0
      ],
      [
        -0.495,
        0.0,
        -0.495
      ],
      [
        0.0,
        0.0,
        -0.7
      ],
      [
        0.495,
        0.0,
        -0.495
      ],
      [
        0.7,
        0.0,
        0.0
      ],
      [
        0.495,
        0.0,
        0.495
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
    0.3333,
    0.3333
  ],
  "speed": [
    6.0,
    6.0
  ],
  "size": [
    0.3,
    0.3
  ],
  "heightOverLife": [
    [
      0.0,
      3.3333
    ],
    [
      1.0,
      6.6667
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0,
        0.2195,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.2195,
        0.0,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
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
      0.8,
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line6"
  },
  "blend": "additive"
});
