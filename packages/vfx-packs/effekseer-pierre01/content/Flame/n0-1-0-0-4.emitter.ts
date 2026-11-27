import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.4 "Light"
export default file.particles({
  "maxParticles": 40,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.8575,
      0.5145
    ]
  },
  "spread": 28.0725,
  "rate": 300.0,
  "duration": 0.1067,
  "looping": false,
  "lifetime": [
    0.7,
    0.8
  ],
  "speed": [
    4.7434,
    12.9035
  ],
  "speedOverLife": [
    [
      0.0,
      2.3953
    ],
    [
      0.2031,
      1.6178
    ],
    [
      0.3906,
      1.0751
    ],
    [
      0.625,
      0.6548
    ],
    [
      0.875,
      0.5225
    ],
    [
      0.8906,
      0.2038
    ],
    [
      0.9062,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    8.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.625
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2845,
        0.0478,
        0.0
      ]
    ],
    [
      0.2667,
      [
        1.0,
        0.2845,
        0.0478,
        0.1098
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2845,
        0.0478,
        0.1098
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2845,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.4423,
        0.1166,
        0.0
      ]
    ],
    [
      0.2667,
      [
        1.0,
        0.4423,
        0.1166,
        0.2039
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.4423,
        0.1166,
        0.2039
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.4423,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
