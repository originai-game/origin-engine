import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0.0 "LineMain"
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
    1.0,
    1.0
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.2222,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
