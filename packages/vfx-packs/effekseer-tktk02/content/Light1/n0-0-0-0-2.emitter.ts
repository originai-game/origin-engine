import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.2 "LineGlow"
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
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    1.875,
    5.625
  ],
  "speedOverLife": [
    [
      0.0,
      2.9785
    ],
    [
      0.1562,
      2.1316
    ],
    [
      0.2969,
      1.4897
    ],
    [
      0.5,
      0.7996
    ],
    [
      0.7031,
      0.3896
    ],
    [
      0.7188,
      0.0808
    ],
    [
      0.7344,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    2.5,
    3.75
  ],
  "heightOverLife": [
    [
      0.0,
      0.3
    ],
    [
      1.0,
      0.3
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
        0.2195,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.2195,
        1.0,
        0.0104,
        0.251
      ]
    ],
    [
      0.5714,
      [
        0.2195,
        1.0,
        0.0104,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5295,
        1.0,
        0.1166,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.5295,
        1.0,
        0.1166,
        0.3765
      ]
    ],
    [
      0.5714,
      [
        0.5295,
        1.0,
        0.1166,
        0.3765
      ]
    ],
    [
      1.0,
      [
        0.5295,
        1.0,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line6"
  },
  "blend": "additive"
});
