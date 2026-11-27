import { file } from '@origingame/engine/author';
// Effekseer node 3.1.0 "衝撃線"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 300
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    8.0,
    16.0
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
    3.0,
    3.0
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
      1.6667
    ],
    [
      1.0,
      1.0
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
      0.1333,
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
