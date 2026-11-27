import { file } from '@origingame/engine/author';
// Effekseer node 0.7.6.0.4 "Light"
export default file.particles({
  "maxParticles": 92,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 14,
      "interval": 0.2
    }
  ],
  "duration": 2.6167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    9.0,
    9.0
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
        1.0,
        0.1636,
        0.0254,
        0.0
      ]
    ],
    [
      0.4,
      [
        1.0,
        0.1636,
        0.0254,
        0.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1636,
        0.0254,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2845,
        0.0781,
        0.0
      ]
    ],
    [
      0.2031,
      [
        1.0,
        0.2845,
        0.0781,
        0.1016
      ]
    ],
    [
      0.4,
      [
        1.0,
        0.2845,
        0.0781,
        0.1506
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2845,
        0.0781,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
