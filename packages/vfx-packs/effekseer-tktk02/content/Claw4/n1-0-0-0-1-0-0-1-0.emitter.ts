import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.1.0.0.1.0 "LineGlow"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.375,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 100
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    5.6445,
    19.1914
  ],
  "speedOverLife": [
    [
      0.0,
      4.6075
    ],
    [
      0.2188,
      1.9913
    ],
    [
      0.3281,
      1.0748
    ],
    [
      0.4375,
      0.4387
    ],
    [
      0.5938,
      0.0164
    ],
    [
      0.7812,
      0.265
    ],
    [
      0.8125,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    3.0104,
    4.5156
  ],
  "heightOverLife": [
    [
      0.0,
      0.5
    ],
    [
      1.0,
      0.5
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
        0.5356,
        1.0,
        0.5356,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.391,
        1.0,
        0.391,
        0.251
      ]
    ],
    [
      0.4062,
      [
        0.2674,
        1.0,
        0.2674,
        0.251
      ]
    ],
    [
      0.6,
      [
        0.1741,
        1.0,
        0.1741,
        0.251
      ]
    ],
    [
      0.7969,
      [
        0.1011,
        1.0,
        0.1011,
        0.1275
      ]
    ],
    [
      1.0,
      [
        0.0478,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7445,
        1.0,
        0.7445,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.6997,
        1.0,
        0.6997,
        0.251
      ]
    ],
    [
      0.6,
      [
        0.6147,
        1.0,
        0.6147,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
