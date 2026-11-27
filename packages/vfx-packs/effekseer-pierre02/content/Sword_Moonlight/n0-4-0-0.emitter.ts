import { file } from '@origingame/engine/author';
// Effekseer node 0.4.0.0 "Light"
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 13,
      "interval": 0.0167
    }
  ],
  "duration": 0.2167,
  "looping": false,
  "lifetime": [
    0.6667,
    0.8333
  ],
  "speed": [
    48.0,
    48.0
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
        0.2195,
        0.1636,
        0.0
      ]
    ],
    [
      0.2667,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0
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
      0.2667,
      [
        0.0478,
        0.3587,
        0.2845,
        0.1255
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.3587,
        0.2845,
        0.1255
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
