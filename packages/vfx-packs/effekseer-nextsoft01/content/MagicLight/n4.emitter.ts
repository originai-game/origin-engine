import { file } from '@origingame/engine/author';
// Effekseer node 4 "maru"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.5,
      2.5,
      2.5
    ]
  },
  "spread": 93.2377,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 35,
      "interval": 0.0167
    }
  ],
  "duration": 0.5833,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    10.6236,
    17.8846
  ],
  "size": [
    1.5,
    1.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1562,
      0.9551
    ],
    [
      0.3281,
      0.8682
    ],
    [
      1.0,
      0.4
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.1094,
      [
        0.198,
        0.9504,
        1.0,
        1.0
      ]
    ],
    [
      0.2188,
      [
        0.1653,
        0.8722,
        1.0,
        1.0
      ]
    ],
    [
      0.625,
      [
        0.0359,
        0.4866,
        1.0,
        1.0
      ]
    ],
    [
      0.8125,
      [
        0.007,
        0.3319,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.2195,
        1.0,
        1.0
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
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Particle01"
  },
  "blend": "additive"
});
