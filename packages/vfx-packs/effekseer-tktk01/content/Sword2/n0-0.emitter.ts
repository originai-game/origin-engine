import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "剣線"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    32.0,
    36.0
  ],
  "speedOverLife": [
    [
      0.0,
      3.6895
    ],
    [
      0.25,
      1.5124
    ],
    [
      0.375,
      0.7867
    ],
    [
      0.5,
      0.3172
    ],
    [
      0.625,
      0.1037
    ],
    [
      0.75,
      0.1464
    ],
    [
      0.875,
      0.4452
    ],
    [
      1.0,
      0.9788
    ]
  ],
  "size": [
    1.5,
    1.5
  ],
  "heightOverLife": [
    [
      0.0,
      1.3333
    ],
    [
      0.0781,
      1.9498
    ],
    [
      0.1562,
      2.4676
    ],
    [
      0.2344,
      2.8998
    ],
    [
      0.3281,
      3.3239
    ],
    [
      0.4688,
      3.814
    ],
    [
      0.8281,
      4.7578
    ],
    [
      1.0,
      5.3333
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
        0.0759,
        0.0759,
        0.1137
      ]
    ],
    [
      0.4375,
      [
        1.0,
        0.0759,
        0.0759,
        0.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0759,
        0.0759,
        0.0
      ]
    ]
  ],
  "rotation": [
    180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line6"
  },
  "blend": "additive"
});
