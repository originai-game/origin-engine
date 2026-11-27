import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0.1 "LineColor"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    3.5,
    10.5
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
    2.0,
    2.0
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
        0.0478,
        0.0478,
        0.2195,
        0.0
      ]
    ],
    [
      0.2222,
      [
        0.0478,
        0.0478,
        0.2195,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.0478,
        0.0478,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0478,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.2195,
        0.5295,
        0.0
      ]
    ],
    [
      0.2222,
      [
        0.2195,
        0.2195,
        0.5295,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.2195,
        0.2195,
        0.5295,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        0.5295,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
