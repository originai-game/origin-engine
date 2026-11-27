import { file } from '@origingame/engine/author';
// Effekseer node 0.6.2.0.0.3.0 "Ray"
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.9998,
      0.0222
    ]
  },
  "spread": 0.3626,
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    17.5071,
    27.5045
  ],
  "speedOverLife": [
    [
      0.0,
      0.9134
    ],
    [
      0.2656,
      1.3914
    ],
    [
      0.3906,
      1.4537
    ],
    [
      0.5312,
      1.3865
    ],
    [
      0.7812,
      0.9413
    ],
    [
      0.7969,
      0.634
    ],
    [
      0.8125,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    16.0,
    32.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.6719,
      0.6419
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
        0.0461,
        0.061,
        0.5356,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.0461,
        0.061,
        0.5356,
        0.3765
      ]
    ],
    [
      0.3333,
      [
        0.0461,
        0.061,
        0.5356,
        0.3765
      ]
    ],
    [
      1.0,
      [
        0.0461,
        0.061,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
