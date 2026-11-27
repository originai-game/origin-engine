import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "random"
export default file.particles({
  "maxParticles": 4096,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.425,
      0.425,
      1.7
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5,
      "cycles": 8,
      "interval": 0.0167
    }
  ],
  "duration": 0.1333,
  "looping": false,
  "lifetime": [
    2.0,
    2.0
  ],
  "speed": [
    0.4417,
    1.7667
  ],
  "size": [
    -0.255,
    0.425
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2674,
        0.2674,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2674,
        0.2674,
        1.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2674,
        0.5668,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2674,
        0.5668,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle01"
  },
  "blend": "additive"
});
