import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "衝撃線"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.1667,
    0.5
  ],
  "speed": [
    12.0,
    24.0
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
      0.5
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      1.0
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
      0.0938,
      [
        1.0,
        1.0,
        1.0,
        0.0878
      ]
    ],
    [
      0.2656,
      [
        1.0,
        1.0,
        1.0,
        0.3274
      ]
    ],
    [
      0.3438,
      [
        1.0,
        1.0,
        1.0,
        0.4189
      ]
    ],
    [
      0.4219,
      [
        1.0,
        1.0,
        1.0,
        0.4791
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.5
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
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/rainnmizu"
  },
  "blend": "additive"
});
