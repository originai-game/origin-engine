import { file } from '@origingame/engine/author';
// Effekseer node 2.1.0 "衝撃線"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 100
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5833,
    0.75
  ],
  "speed": [
    1.5,
    7.5
  ],
  "speedOverLife": [
    [
      0.0,
      3.6937
    ],
    [
      0.25,
      1.6792
    ],
    [
      0.375,
      0.9618
    ],
    [
      0.5,
      0.4506
    ],
    [
      0.625,
      0.1454
    ],
    [
      0.75,
      0.0463
    ],
    [
      0.875,
      0.1533
    ],
    [
      1.0,
      0.4534
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
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      1.5
    ],
    [
      1.0,
      1.5
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
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
      0.075,
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
        1.0
      ]
    ]
  ],
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/rainnmizu"
  },
  "blend": "additive"
});
