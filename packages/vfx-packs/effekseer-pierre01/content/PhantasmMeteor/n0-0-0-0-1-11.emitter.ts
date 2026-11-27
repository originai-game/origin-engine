import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.11 "DarkEye"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    1.2,
    1.2
  ],
  "speed": [
    6.113,
    6.113
  ],
  "speedOverLife": [
    [
      0.0,
      1.9822
    ],
    [
      0.2344,
      1.9821
    ],
    [
      0.2656,
      2.2218
    ],
    [
      0.7969,
      2.2218
    ],
    [
      0.8125,
      1.7815
    ],
    [
      0.8281,
      7.8488
    ],
    [
      1.0,
      7.8488
    ]
  ],
  "size": [
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2188,
      0.8564
    ],
    [
      0.4531,
      0.7276
    ],
    [
      0.7031,
      0.6132
    ],
    [
      1.0,
      0.5
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
      0.2083,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.5833,
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
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "subtractive"
});
