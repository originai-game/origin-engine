import { file } from '@origingame/engine/author';
// Effekseer node 2.0.1.0.0.1 "LineColor"
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
    0.5,
    0.5
  ],
  "speed": [
    1.3125,
    3.9375
  ],
  "speedOverLife": [
    [
      0.0,
      4.4254
    ],
    [
      0.2188,
      1.9936
    ],
    [
      0.3281,
      1.1232
    ],
    [
      0.4375,
      0.5008
    ],
    [
      0.6094,
      0.024
    ],
    [
      0.8125,
      0.2502
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
      0.1667,
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
      0.1667,
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
