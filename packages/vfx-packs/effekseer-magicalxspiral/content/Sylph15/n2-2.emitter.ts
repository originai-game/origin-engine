import { file } from '@origingame/engine/author';
// Effekseer node 2.2 "Node"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "sphere",
    "radius": 0.0
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.1
    ],
    [
      1.0,
      0.1
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.5356,
        0.0,
        0.3765
      ]
    ],
    [
      0.4219,
      [
        0.3808,
        0.5356,
        0.0,
        0.3765
      ]
    ],
    [
      0.8333,
      [
        0.2611,
        0.5356,
        0.0,
        0.3765
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.5356,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.0,
        0.3765
      ]
    ],
    [
      0.2031,
      [
        0.7909,
        0.5356,
        0.0,
        0.3765
      ]
    ],
    [
      0.4062,
      [
        0.6082,
        0.5356,
        0.0,
        0.3765
      ]
    ],
    [
      0.625,
      [
        0.4403,
        0.5356,
        0.0,
        0.3765
      ]
    ],
    [
      0.8333,
      [
        0.3074,
        0.5356,
        0.0,
        0.3765
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.5356,
        0.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -90.0,
    -90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Thunder2"
  },
  "sheet": {
    "columns": 2,
    "rows": 4,
    "frames": 8,
    "mode": {
      "kind": "fps",
      "fps": 20.0
    }
  },
  "blend": "additive"
});
