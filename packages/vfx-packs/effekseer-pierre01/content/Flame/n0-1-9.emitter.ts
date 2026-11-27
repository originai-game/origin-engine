import { file } from '@origingame/engine/author';
// Effekseer node 0.1.9 "Flash"
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
    32.0,
    32.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.125
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7519,
        0.3587,
        0.0
      ]
    ],
    [
      0.0781,
      [
        1.0,
        0.7519,
        0.3587,
        0.3704
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.7519,
        0.3587,
        0.5625
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.7519,
        0.3587,
        0.7143
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.3587,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
