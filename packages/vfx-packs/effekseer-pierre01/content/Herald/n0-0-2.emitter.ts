import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2 "Ray"
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
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    7.7969,
    7.7969
  ],
  "sizeOverLife": [
    [
      0.0,
      0.513
    ],
    [
      0.125,
      0.6826
    ],
    [
      0.25,
      0.816
    ],
    [
      0.5,
      0.9747
    ],
    [
      0.625,
      1.0
    ],
    [
      0.75,
      0.9892
    ],
    [
      0.875,
      0.9423
    ],
    [
      1.0,
      0.8593
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.1283
    ],
    [
      0.125,
      0.1517
    ],
    [
      0.25,
      0.1743
    ],
    [
      0.5,
      0.2168
    ],
    [
      0.625,
      0.2366
    ],
    [
      0.75,
      0.2556
    ],
    [
      0.875,
      0.2737
    ],
    [
      1.0,
      0.2908
    ]
  ],
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
      0.1667,
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
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Star"
  },
  "blend": "additive"
});
