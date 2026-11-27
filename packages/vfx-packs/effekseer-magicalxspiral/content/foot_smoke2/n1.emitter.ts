import { file } from '@origingame/engine/author';
// Effekseer node 1 "Stars"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2,
      0.2,
      0.2
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.3637,
    1.4549
  ],
  "size": [
    0.3,
    0.3
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.5356,
        1.0,
        0.0
      ]
    ],
    [
      0.06,
      [
        0.5356,
        0.5356,
        1.0,
        0.1255
      ]
    ],
    [
      0.2,
      [
        0.5356,
        0.5356,
        1.0,
        0.1255
      ]
    ],
    [
      0.4688,
      [
        0.5356,
        0.5356,
        1.0,
        0.0621
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.5356,
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.06,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.2,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.3281,
      [
        1.0,
        1.0,
        1.0,
        0.2675
      ]
    ],
    [
      0.4688,
      [
        1.0,
        1.0,
        1.0,
        0.1862
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
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/star"
  },
  "blend": "additive"
});
