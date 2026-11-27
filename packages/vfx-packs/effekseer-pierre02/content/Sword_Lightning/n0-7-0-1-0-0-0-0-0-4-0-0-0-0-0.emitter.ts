import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.0.0.0.4.0.0.0.0.0 "Light"
export default file.particles({
  "maxParticles": 1210,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 17599.7051,
  "duration": 0.09,
  "looping": false,
  "lifetime": [
    0.0333,
    0.0667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    12.0,
    20.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2188,
      0.7577
    ],
    [
      0.4531,
      0.5403
    ],
    [
      0.7031,
      0.3472
    ],
    [
      1.0,
      0.1562
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0461,
        0.0511,
        0.5356,
        0.0353
      ]
    ],
    [
      1.0,
      [
        0.0461,
        0.0511,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
