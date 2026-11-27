import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.4 "Stars"
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
    2.1667,
    2.8333
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
        0.4423,
        0.2195,
        0.5295,
        0.498
      ]
    ],
    [
      0.8667,
      [
        0.4423,
        0.2195,
        0.5295,
        0.498
      ]
    ],
    [
      1.0,
      [
        0.4423,
        0.2195,
        0.5295,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.687,
        0.5295,
        0.7445,
        0.749
      ]
    ],
    [
      0.8667,
      [
        0.687,
        0.5295,
        0.7445,
        0.749
      ]
    ],
    [
      1.0,
      [
        0.687,
        0.5295,
        0.7445,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Hard"
  },
  "blend": "additive"
});
