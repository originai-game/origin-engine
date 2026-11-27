import { file } from '@origingame/engine/author';
// Effekseer node 0.7.2.0 "Flash1"
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
      "count": 1,
      "cycles": 3,
      "interval": 0.0583
    }
  ],
  "duration": 0.1333,
  "looping": false,
  "lifetime": [
    0.2,
    0.2
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    144.0,
    144.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.25
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5295,
        0.673,
        1.0,
        0.0
      ]
    ],
    [
      0.0833,
      [
        0.5295,
        0.673,
        1.0,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.5295,
        0.673,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5295,
        0.673,
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
