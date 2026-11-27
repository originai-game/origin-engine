import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.1.0 "LineColor"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    5.3036,
    15.9107
  ],
  "speedOverLife": [
    [
      0.0,
      5.2564
    ],
    [
      0.2031,
      2.1303
    ],
    [
      0.2969,
      1.1396
    ],
    [
      0.4062,
      0.3723
    ],
    [
      0.5312,
      0.0079
    ],
    [
      0.6875,
      0.3212
    ],
    [
      0.7188,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    3.0938,
    3.7812
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
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.2,
      [
        1.0,
        0.1741,
        0.0379,
        0.251
      ]
    ],
    [
      0.5938,
      [
        1.0,
        0.1011,
        0.022,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.2,
      [
        1.0,
        0.4602,
        0.1741,
        0.251
      ]
    ],
    [
      0.5938,
      [
        1.0,
        0.3297,
        0.1011,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
