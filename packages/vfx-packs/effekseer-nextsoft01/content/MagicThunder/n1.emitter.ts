import { file } from '@origingame/engine/author';
// Effekseer node 1 "雷"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    40.0,
    40.0
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
    4.38,
    4.38
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6849
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      6.8493
    ],
    [
      1.0,
      7.1918
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      -1.0,
      0.0
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
      0.7188,
      [
        1.0,
        1.0,
        1.0,
        0.3879
      ]
    ],
    [
      0.8594,
      [
        1.0,
        1.0,
        1.0,
        0.211
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
  "gravity": [
    0.0,
    -3.6,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Thunder10"
  },
  "blend": "alpha"
});
