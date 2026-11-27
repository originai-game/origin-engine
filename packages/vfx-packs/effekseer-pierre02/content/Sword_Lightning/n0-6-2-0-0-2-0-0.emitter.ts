import { file } from '@origingame/engine/author';
// Effekseer node 0.6.2.0.0.2.0.0 "SparkLight"
export default file.particles({
  "maxParticles": 50,
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
    0.0667,
    0.0667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.75,
    1.25
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.6
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
      0.25,
      [
        0.0478,
        0.1801,
        1.0,
        0.0376
      ]
    ],
    [
      0.5,
      [
        0.0478,
        0.1801,
        1.0,
        0.0502
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
