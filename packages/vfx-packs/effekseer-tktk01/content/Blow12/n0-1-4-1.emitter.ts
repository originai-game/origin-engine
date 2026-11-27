import { file } from '@origingame/engine/author';
// Effekseer node 0.1.4.1 "ラインカラー"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    3.0,
    3.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1324
    ],
    [
      0.2656,
      1.4047
    ],
    [
      0.5,
      0.8969
    ],
    [
      0.75,
      0.5051
    ],
    [
      1.0,
      0.2705
    ]
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.7364
    ],
    [
      0.1719,
      0.4882
    ],
    [
      0.2656,
      0.3051
    ],
    [
      0.375,
      0.1605
    ],
    [
      0.4688,
      0.0844
    ],
    [
      0.5781,
      0.0375
    ],
    [
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      2.0
    ],
    [
      0.0781,
      2.5273
    ],
    [
      0.1719,
      3.0236
    ],
    [
      0.2656,
      3.3899
    ],
    [
      0.375,
      3.6791
    ],
    [
      0.4688,
      3.8313
    ],
    [
      0.5781,
      3.9249
    ],
    [
      1.0,
      4.0
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
        0.0,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.0
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
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
