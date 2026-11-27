import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.9 "Ray"
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
    0.6,
    0.6
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    12.835,
    12.835
  ],
  "sizeOverLife": [
    [
      0.0,
      0.7791
    ],
    [
      0.5,
      0.9211
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.1948
    ],
    [
      0.5,
      0.2132
    ],
    [
      1.0,
      0.2
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
      0.0833,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.3333,
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
