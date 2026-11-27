import { file } from '@origingame/engine/author';
// Effekseer node 0.5.0.0 "ライン色付け"
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
    0.75,
    0.75
  ],
  "speed": [
    8.32,
    8.32
  ],
  "speedOverLife": [
    [
      0.0,
      3.8159
    ],
    [
      0.1406,
      2.4721
    ],
    [
      0.2812,
      1.5066
    ],
    [
      0.4062,
      1.0032
    ],
    [
      0.4688,
      0.8767
    ],
    [
      0.5469,
      0.836
    ],
    [
      0.5625,
      0.0466
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    2.6667,
    2.6667
  ],
  "heightOverLife": [
    [
      0.0,
      0.5
    ],
    [
      1.0,
      0.5
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
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.7529
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
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.7529
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
