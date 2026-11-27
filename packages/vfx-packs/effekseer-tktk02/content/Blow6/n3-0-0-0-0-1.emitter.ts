import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.0.0.1 "LineColor"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.3125,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.8203,
    2.4609
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
      0.4388
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
    2.9167,
    2.9167
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
        0.2195,
        0.0478,
        0.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.2195,
        0.0478,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
        0.0,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.2195,
        0.2195,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.5295,
        0.2195,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.2195,
        0.2195,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
