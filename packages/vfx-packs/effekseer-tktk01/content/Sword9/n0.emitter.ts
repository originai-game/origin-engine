import { file } from '@origingame/engine/author';
// Effekseer node 0 "剣線"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    16.8,
    16.8
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
    3.0,
    3.0
  ],
  "heightOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.7375
    ],
    [
      0.1719,
      0.493
    ],
    [
      0.2656,
      0.3153
    ],
    [
      0.375,
      0.1779
    ],
    [
      0.5625,
      0.0695
    ],
    [
      0.875,
      0.0366
    ],
    [
      1.0,
      0.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      -1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3112,
        0.3112,
        0.7843
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3112,
        0.3112,
        0.7843
      ]
    ]
  ],
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Thunder1"
  },
  "blend": "additive"
});
