import { file } from '@origingame/engine/author';
// Effekseer node 1.0.2.0.0.0 "shield"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.434,
        0.0
      ],
      [
        -0.3069,
        0.3069,
        0.0
      ],
      [
        -0.434,
        0.0,
        0.0
      ],
      [
        -0.3069,
        -0.3069,
        0.0
      ],
      [
        0.0,
        -0.434,
        0.0
      ],
      [
        0.3069,
        -0.3069,
        0.0
      ],
      [
        0.434,
        0.0,
        0.0
      ],
      [
        0.3069,
        0.3069,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 6
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    1.3333,
    1.3333
  ],
  "speedOverLife": [
    [
      0.0,
      4.0
    ],
    [
      0.1875,
      4.0
    ],
    [
      0.2031,
      1.9
    ],
    [
      0.2188,
      1.0
    ],
    [
      0.3906,
      1.0
    ],
    [
      0.4062,
      0.1
    ],
    [
      0.4219,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.3333,
    0.3333
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
        0.0
      ]
    ],
    [
      0.0938,
      [
        1.0,
        1.0,
        1.0,
        0.7059
      ]
    ],
    [
      0.1094,
      [
        1.0,
        1.0,
        1.0,
        0.7761
      ]
    ],
    [
      0.1875,
      [
        1.0,
        1.0,
        1.0,
        0.9691
      ]
    ],
    [
      0.2031,
      [
        1.0,
        1.0,
        1.0,
        0.9766
      ]
    ],
    [
      0.2969,
      [
        1.0,
        1.0,
        1.0,
        0.2744
      ]
    ],
    [
      0.4062,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "angularVelocity": [
    108.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/shield1"
  },
  "blend": "additive"
});
