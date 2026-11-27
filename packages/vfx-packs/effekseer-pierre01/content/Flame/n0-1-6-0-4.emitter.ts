import { file } from '@origingame/engine/author';
// Effekseer node 0.1.6.0.4 "Light"
export default file.particles({
  "maxParticles": 75,
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
    4.6667,
    4.6667
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
        0.2845,
        0.0478,
        0.0
      ]
    ],
    [
      0.4,
      [
        1.0,
        0.2845,
        0.0478,
        0.0376
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
      0.2031,
      [
        1.0,
        0.4423,
        0.1166,
        0.127
      ]
    ],
    [
      0.4,
      [
        1.0,
        0.4423,
        0.1166,
        0.1882
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
