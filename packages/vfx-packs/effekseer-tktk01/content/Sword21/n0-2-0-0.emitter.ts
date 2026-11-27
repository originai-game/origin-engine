import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0 "色"
export default file.particles({
  "maxParticles": 250,
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
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    1.8333,
    8.25
  ],
  "speedOverLife": [
    [
      0.0,
      12.0339
    ],
    [
      0.0625,
      6.7405
    ],
    [
      0.1094,
      3.6016
    ],
    [
      0.1562,
      1.4393
    ],
    [
      0.2031,
      0.2536
    ],
    [
      0.2969,
      0.5236
    ],
    [
      0.3125,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.375,
    1.375
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
        0.0,
        0.0,
        0.0,
        0.0
      ]
    ],
    [
      0.02,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0
      ]
    ],
    [
      0.02,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
