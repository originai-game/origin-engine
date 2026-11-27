import { file } from '@origingame/engine/author';
// Effekseer node 13.0 "Node"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        -1.25
      ],
      [
        -0.8839,
        0.0,
        -0.8839
      ],
      [
        -1.25,
        0.0,
        0.0
      ],
      [
        -0.8839,
        0.0,
        0.8839
      ],
      [
        0.0,
        0.0,
        1.25
      ],
      [
        0.8839,
        0.0,
        0.8839
      ],
      [
        1.25,
        0.0,
        0.0
      ],
      [
        0.8839,
        0.0,
        -0.8839
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.05
    }
  ],
  "duration": 0.4667,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    3.0,
    9.0
  ],
  "size": [
    0.5,
    0.5
  ],
  "heightOverLife": [
    [
      0.0,
      0.0
    ],
    [
      1.0,
      4.0
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
        0.9658,
        0.0478,
        0.0478,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.9658,
        0.0478,
        0.0478,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.9658,
        0.0478,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.9828,
        0.5356,
        0.5356,
        0.502
      ]
    ],
    [
      0.8,
      [
        0.9828,
        0.5356,
        0.5356,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.9828,
        0.5356,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line6"
  },
  "blend": "additive"
});
