import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0.0.1 "LineColor"
export default file.particles({
  "maxParticles": 500,
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
      "count": 50
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    3.1875,
    9.5625
  ],
  "speedOverLife": [
    [
      0.0,
      3.6952
    ],
    [
      0.25,
      1.7411
    ],
    [
      0.375,
      1.0269
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      0.1608
    ],
    [
      0.75,
      0.0091
    ],
    [
      0.875,
      0.0448
    ],
    [
      1.0,
      0.2582
    ]
  ],
  "size": [
    1.8594,
    2.3906
  ],
  "heightOverLife": [
    [
      0.0,
      0.625
    ],
    [
      1.0,
      0.625
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
        1.0
      ]
    ],
    [
      0.4062,
      [
        0.2674,
        1.0,
        0.2674,
        1.0
      ]
    ],
    [
      0.6,
      [
        0.1741,
        1.0,
        0.1741,
        1.0
      ]
    ],
    [
      0.7969,
      [
        0.1011,
        1.0,
        0.1011,
        0.5078
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
        0.6537,
        1.0,
        0.6997,
        1.0
      ]
    ],
    [
      0.6,
      [
        0.4913,
        1.0,
        0.6147,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.3537,
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
