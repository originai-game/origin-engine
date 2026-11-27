import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0 "レンズフレアglow"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        -5.25,
        0.0
      ],
      [
        3.7123,
        -3.7123,
        0.0
      ],
      [
        5.25,
        0.0,
        0.0
      ],
      [
        3.7123,
        3.7123,
        0.0
      ],
      [
        0.0,
        5.25,
        0.0
      ],
      [
        -3.7123,
        3.7123,
        0.0
      ],
      [
        -5.25,
        0.0,
        0.0
      ],
      [
        -3.7123,
        -3.7123,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 10
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    9.35,
    9.35
  ],
  "speedOverLife": [
    [
      0.0,
      2.1308
    ],
    [
      0.25,
      1.3814
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      0.5423
    ],
    [
      1.0,
      0.4657
    ]
  ],
  "size": [
    6.375,
    6.375
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2031,
      0.8772
    ],
    [
      0.4219,
      0.7873
    ],
    [
      0.6562,
      0.7253
    ],
    [
      1.0,
      0.6667
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.6667
    ],
    [
      0.2031,
      0.6667
    ],
    [
      0.4219,
      0.6667
    ],
    [
      0.6562,
      0.6667
    ],
    [
      1.0,
      0.6667
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
        0.0478,
        0.0478,
        0.0478,
        0.0
      ]
    ],
    [
      0.0333,
      [
        0.0478,
        0.0478,
        0.0478,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0478,
        0.0478,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3537,
        0.3537,
        0.3537,
        0.0
      ]
    ],
    [
      0.0333,
      [
        0.3537,
        0.3537,
        0.3537,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.3537,
        0.3537,
        0.3537,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/actionlines"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-1-0-0.effect"
  }
});
