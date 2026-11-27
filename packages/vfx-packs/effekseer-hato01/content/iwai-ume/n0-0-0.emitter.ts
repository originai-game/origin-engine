import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "Node"
export default file.particles({
  "maxParticles": 628,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.21,
      0.18,
      0.33
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2100,
      "interval": 0.0167
    }
  ],
  "duration": 35.0,
  "looping": false,
  "lifetime": [
    0.8333,
    7.5
  ],
  "speed": [
    0.2271,
    0.9085
  ],
  "size": [
    1.8,
    1.08
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.325,
        0.325,
        0.0
      ]
    ],
    [
      0.0728,
      [
        1.0,
        0.325,
        0.325,
        1.0
      ]
    ],
    [
      0.8276,
      [
        1.0,
        0.325,
        0.325,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.325,
        0.325,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5054,
        0.5054,
        0.0
      ]
    ],
    [
      0.0728,
      [
        1.0,
        0.5054,
        0.5054,
        1.0
      ]
    ],
    [
      0.8276,
      [
        1.0,
        0.5054,
        0.5054,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5054,
        0.5054,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-hato01/textures/hana-anime"
  },
  "sheet": {
    "columns": 2,
    "rows": 2,
    "frames": 4,
    "mode": {
      "kind": "fps",
      "fps": 1.5
    }
  },
  "blend": "additive"
});
