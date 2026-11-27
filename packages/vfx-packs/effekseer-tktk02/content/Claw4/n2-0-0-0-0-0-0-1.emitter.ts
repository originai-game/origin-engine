import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.0.0.0.1 "LineColor"
export default file.particles({
  "maxParticles": 125,
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
    0.75,
    0.75
  ],
  "speed": [
    2.6563,
    12.2188
  ],
  "speedOverLife": [
    [
      0.0,
      5.5153
    ],
    [
      0.2031,
      2.1086
    ],
    [
      0.2969,
      1.0615
    ],
    [
      0.3906,
      0.3703
    ],
    [
      0.5156,
      0.0025
    ],
    [
      0.6562,
      0.3451
    ],
    [
      0.6875,
      0.0
    ],
    [
      1.0,
      0.0
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
        1.0,
        0.5356,
        0.5356,
        0.0
      ]
    ],
    [
      0.1111,
      [
        1.0,
        0.4522,
        0.4522,
        1.0
      ]
    ],
    [
      0.4375,
      [
        1.0,
        0.2508,
        0.2508,
        1.0
      ]
    ],
    [
      0.7778,
      [
        1.0,
        0.1073,
        0.1073,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
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
        1.0,
        0.7445,
        0.7445,
        0.0
      ]
    ],
    [
      0.1111,
      [
        1.0,
        0.7195,
        0.7195,
        1.0
      ]
    ],
    [
      0.7778,
      [
        1.0,
        0.5788,
        0.5788,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
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
