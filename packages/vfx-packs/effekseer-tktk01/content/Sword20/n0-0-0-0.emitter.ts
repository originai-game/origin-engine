import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "色付け"
export default file.particles({
  "maxParticles": 70,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        2.0,
        0.0
      ],
      [
        -1.4142,
        1.4142,
        0.0
      ],
      [
        -2.0,
        0.0,
        0.0
      ],
      [
        -1.4142,
        -1.4142,
        0.0
      ],
      [
        0.0,
        -2.0,
        0.0
      ],
      [
        1.4142,
        -1.4142,
        0.0
      ],
      [
        2.0,
        0.0,
        0.0
      ],
      [
        1.4142,
        1.4142,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 7
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    23.4,
    39.0
  ],
  "speedOverLife": [
    [
      0.0,
      4.4232
    ],
    [
      0.2188,
      1.9179
    ],
    [
      0.4219,
      0.5165
    ],
    [
      0.5781,
      0.0785
    ],
    [
      0.8125,
      0.4649
    ],
    [
      0.8281,
      0.4408
    ],
    [
      0.8438,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    3.25,
    3.25
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
        0.0,
        0.0,
        0.0,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.7529
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.7529
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
