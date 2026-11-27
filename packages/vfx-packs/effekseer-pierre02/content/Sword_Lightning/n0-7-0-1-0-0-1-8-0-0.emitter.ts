import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.8.0.0 "SparkLight"
export default file.particles({
  "maxParticles": 867,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 33,
      "cycles": 60,
      "interval": 0.0167
    }
  ],
  "duration": 1.0,
  "looping": false,
  "lifetime": [
    0.1,
    0.1
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.05,
    1.75
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
        0.1062,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.0478,
        0.1062,
        1.0,
        0.0627
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.1062,
        1.0,
        0.1004
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.1062,
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
