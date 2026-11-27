import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.3.7 "Light"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    80.0,
    80.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.4219,
      0.82
    ],
    [
      0.625,
      0.7037
    ],
    [
      0.8281,
      0.5285
    ],
    [
      1.0,
      0.3125
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.5418,
        1.0,
        0.502
      ]
    ],
    [
      0.5,
      [
        0.5295,
        0.5418,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.5418,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
