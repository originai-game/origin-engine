import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.0.0.1 "LineColor"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.125,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.525,
    1.575
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
    1.1667,
    1.1667
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
        0.0104,
        0.0023,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.0104,
        0.0023,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.0023,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.1166,
        0.0254,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
