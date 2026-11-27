import { file } from '@origingame/engine/author';
// Effekseer node 0.10.0.0.5.0.1 "Light"
export default file.particles({
  "maxParticles": 77,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.001,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 4,
      "interval": 0.1
    }
  ],
  "duration": 0.3167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
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
      0.24,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0941
      ]
    ],
    [
      0.4,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0941
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
      0.24,
      [
        0.0478,
        0.3587,
        0.2845,
        0.2824
      ]
    ],
    [
      0.4,
      [
        0.0478,
        0.3587,
        0.2845,
        0.2824
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
