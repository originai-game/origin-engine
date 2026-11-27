import { file } from '@origingame/engine/author';
// Effekseer node 4 "剣"
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 9,
      "interval": 0.1
    }
  ],
  "duration": 0.8167,
  "looping": false,
  "lifetime": [
    0.9167,
    0.9167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.0,
    2.0
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
        0.2195,
        0.0,
        1.0
      ]
    ],
    [
      0.2031,
      [
        1.0,
        0.3288,
        0.0066,
        1.0
      ]
    ],
    [
      0.4062,
      [
        1.0,
        0.4624,
        0.0303,
        1.0
      ]
    ],
    [
      0.6094,
      [
        1.0,
        0.6213,
        0.0738,
        1.0
      ]
    ],
    [
      0.8182,
      [
        1.0,
        0.8115,
        0.1412,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.2195,
        1.0
      ]
    ],
    [
      0.2031,
      [
        1.0,
        1.0,
        0.3288,
        1.0
      ]
    ],
    [
      0.4062,
      [
        1.0,
        1.0,
        0.4624,
        1.0
      ]
    ],
    [
      0.6094,
      [
        1.0,
        1.0,
        0.6213,
        1.0
      ]
    ],
    [
      0.8182,
      [
        1.0,
        1.0,
        0.8115,
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
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Sword002"
  },
  "blend": "additive"
});
