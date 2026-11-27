import { file } from '@origingame/engine/author';
// Effekseer node 0.11.1.1.0.4.0.1 "Light"
export default file.particles({
  "maxParticles": 1414,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 32,
      "cycles": 2,
      "interval": 0.3
    }
  ],
  "duration": 0.3167,
  "looping": false,
  "lifetime": [
    1.3333,
    1.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.0,
    0.0
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
        0.2195,
        0.1636,
        0.0
      ]
    ],
    [
      0.15,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0314
      ]
    ],
    [
      0.625,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0314
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.3587,
        0.2845,
        0.0
      ]
    ],
    [
      0.15,
      [
        0.0478,
        0.3587,
        0.2845,
        0.1569
      ]
    ],
    [
      0.625,
      [
        0.0478,
        0.3587,
        0.2845,
        0.1569
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.3587,
        0.2845,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
