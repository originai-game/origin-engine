import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "Stars"
export default file.particles({
  "maxParticles": 90,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.9375,
      3.9375,
      3.9375
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 6
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    1.8333,
    2.5
  ],
  "speed": [
    0.1169,
    0.4677
  ],
  "size": [
    0.135,
    0.135
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.3787,
        0.0104,
        0.498
      ]
    ],
    [
      0.8462,
      [
        1.0,
        0.3787,
        0.0104,
        0.498
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3787,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5605,
        0.0781,
        0.749
      ]
    ],
    [
      0.8462,
      [
        1.0,
        0.5605,
        0.0781,
        0.749
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5605,
        0.0781,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Hard"
  },
  "blend": "additive"
});
