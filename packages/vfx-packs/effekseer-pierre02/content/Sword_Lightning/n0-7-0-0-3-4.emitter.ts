import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.3.4 "Flash"
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
    0.3,
    0.3
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
      0.1875
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
        0.548,
        0.5295,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.548,
        0.5295,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.548,
        0.5295,
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
