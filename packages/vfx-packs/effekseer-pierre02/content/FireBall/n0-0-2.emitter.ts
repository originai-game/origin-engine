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
      "count": 1,
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.5167,
    0.5167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    14.524,
    14.524
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1377
    ],
    [
      0.25,
      0.3905
    ],
    [
      0.5,
      0.6185
    ],
    [
      0.75,
      0.8216
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.0689
    ],
    [
      0.25,
      0.1189
    ],
    [
      0.5,
      0.1441
    ],
    [
      0.75,
      0.1445
    ],
    [
      1.0,
      0.1201
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.3871,
      [
        1.0,
        0.2195,
        0.0478,
        1.0
      ]
    ],
    [
      0.4194,
      [
        1.0,
        0.2195,
        0.0478,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
