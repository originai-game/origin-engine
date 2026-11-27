import { file } from '@origingame/engine/author';
// Effekseer node 0.1.4 "Flash"
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
      "interval": 0.0417
    }
  ],
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    0.3,
    0.3
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    54.0,
    54.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.25
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
        0.4647,
        0.0356,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.4647,
        0.0356,
        0.5647
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.4647,
        0.0356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
