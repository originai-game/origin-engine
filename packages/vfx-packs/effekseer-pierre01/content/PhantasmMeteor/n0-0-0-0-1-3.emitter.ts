import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.3 "Light"
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
    1.25,
    1.25
  ],
  "speed": [
    6.113,
    6.113
  ],
  "speedOverLife": [
    [
      0.0,
      1.9724
    ],
    [
      0.2188,
      1.9724
    ],
    [
      0.2344,
      1.9967
    ],
    [
      0.25,
      2.2109
    ],
    [
      0.7656,
      2.2109
    ],
    [
      0.7812,
      1.2084
    ],
    [
      0.7969,
      7.8103
    ],
    [
      1.0,
      7.8103
    ]
  ],
  "size": [
    26.8164,
    26.8164
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0932
    ],
    [
      0.125,
      0.4736
    ],
    [
      0.25,
      0.7515
    ],
    [
      0.375,
      0.9269
    ],
    [
      0.5,
      1.0
    ],
    [
      0.625,
      0.9706
    ],
    [
      0.75,
      0.8389
    ],
    [
      1.0,
      0.268
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
      0.0667,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      0.6,
      [
        1.0,
        1.0,
        1.0,
        0.1255
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
