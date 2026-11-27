import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0 "Node"
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.2,
    0.2
  ],
  "speed": [
    15.0,
    15.0
  ],
  "size": [
    1.5,
    1.5
  ],
  "heightOverLife": [
    [
      0.0,
      0.6667
    ],
    [
      1.0,
      0.6667
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
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.0833,
      [
        1.0,
        0.0478,
        1.0,
        0.502
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.0478,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        1.0,
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
        1.0,
        0.0
      ]
    ],
    [
      0.0833,
      [
        1.0,
        0.5356,
        1.0,
        0.502
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.5356,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line1"
  },
  "blend": "additive"
});
