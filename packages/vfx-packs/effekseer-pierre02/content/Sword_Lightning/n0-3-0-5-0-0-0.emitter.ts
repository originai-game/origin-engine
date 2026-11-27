import { file } from '@origingame/engine/author';
// Effekseer node 0.3.0.5.0.0.0 "SparkLight"
export default file.particles({
  "maxParticles": 65,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 50,
      "interval": 0.0167
    }
  ],
  "duration": 0.8333,
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
    0.0,
    0.0
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
        0.0373
      ]
    ],
    [
      0.6667,
      [
        0.0478,
        0.1801,
        1.0,
        0.0376
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
