import { file } from '@origingame/engine/author';
// Effekseer node 3 "雷"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.2,
      0.001
    ]
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
    0.0,
    0.0
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
      1.0,
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
      0.0156,
      [
        1.0,
        1.0,
        1.0,
        0.2217
      ]
    ],
    [
      0.0312,
      [
        1.0,
        1.0,
        1.0,
        0.3916
      ]
    ],
    [
      0.0469,
      [
        1.0,
        1.0,
        1.0,
        0.5207
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
