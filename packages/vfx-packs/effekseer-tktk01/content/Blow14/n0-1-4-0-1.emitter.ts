import { file } from '@origingame/engine/author';
// Effekseer node 0.1.4.0.1 "ラインカラー"
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
    1.0,
    1.0
  ],
  "speed": [
    2.5,
    2.5
  ],
  "speedOverLife": [
    [
      0.0,
      3.6952
    ],
    [
      0.25,
      1.7411
    ],
    [
      0.375,
      1.0269
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      0.1608
    ],
    [
      0.75,
      0.0091
    ],
    [
      0.875,
      0.0448
    ],
    [
      1.0,
      0.2582
    ]
  ],
  "size": [
    3.0,
    3.0
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
      1.3333
    ],
    [
      0.0781,
      1.6849
    ],
    [
      0.1719,
      2.0157
    ],
    [
      0.2656,
      2.2599
    ],
    [
      0.375,
      2.4527
    ],
    [
      0.4688,
      2.5542
    ],
    [
      0.5781,
      2.6166
    ],
    [
      1.0,
      2.6667
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
