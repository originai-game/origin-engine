import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.3.6 "Ray"
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
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    111.08,
    111.08
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8642
    ],
    [
      0.5,
      0.9429
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.2881
    ],
    [
      0.5,
      0.3668
    ],
    [
      1.0,
      0.4238
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5732,
        0.5295,
        1.0,
        0.0
      ]
    ],
    [
      0.3,
      [
        0.5732,
        0.5295,
        1.0,
        1.0
      ]
    ],
    [
      0.55,
      [
        0.5732,
        0.5295,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.5732,
        0.5295,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
