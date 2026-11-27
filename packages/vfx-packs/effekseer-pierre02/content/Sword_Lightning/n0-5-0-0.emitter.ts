import { file } from '@origingame/engine/author';
// Effekseer node 0.5.0.0 "Light"
export default file.particles({
  "maxParticles": 33,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2744,
      3.5672,
      0.001
    ]
  },
  "spread": 98.0495,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 2,
      "interval": 0.0333
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6667,
    0.8333
  ],
  "speed": [
    2.1377,
    10.6887
  ],
  "speedOverLife": [
    [
      0.0,
      0.4816
    ],
    [
      0.25,
      0.851
    ],
    [
      0.5,
      1.0909
    ],
    [
      0.75,
      1.1944
    ],
    [
      1.0,
      1.1632
    ]
  ],
  "size": [
    32.0,
    32.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
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
        0.0478,
        0.1636,
        0.8671,
        0.0
      ]
    ],
    [
      0.2667,
      [
        0.0478,
        0.1636,
        0.8671,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.1636,
        0.8671,
        0.0
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.1636,
        0.8671,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.2845,
        0.9323,
        0.0
      ]
    ],
    [
      0.2667,
      [
        0.0478,
        0.2845,
        0.9323,
        0.1255
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.2845,
        0.9323,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2845,
        0.9323,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
