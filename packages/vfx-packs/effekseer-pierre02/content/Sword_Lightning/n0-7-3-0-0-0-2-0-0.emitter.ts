import { file } from '@origingame/engine/author';
// Effekseer node 0.7.3.0.0.0.2.0.0 "SparkLight"
export default file.particles({
  "maxParticles": 120,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 60,
      "interval": 0.0167
    }
  ],
  "duration": 1.0,
  "looping": false,
  "lifetime": [
    0.05,
    0.05
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.6875,
    2.8125
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.7
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.1801,
        1.0,
        0.0
      ]
    ],
    [
      0.3281,
      [
        0.0478,
        0.1801,
        1.0,
        0.0249
      ]
    ],
    [
      0.6667,
      [
        0.0478,
        0.1801,
        1.0,
        0.0251
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.1801,
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
