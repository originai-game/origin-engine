import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.3 "Ray"
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 25,
      "interval": 0.0917
    }
  ],
  "duration": 2.2167,
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
    27.77,
    27.77
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
      0.144
    ],
    [
      0.5,
      0.1867
    ],
    [
      1.0,
      0.2078
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
      0.3,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.55,
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
