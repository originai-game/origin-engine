import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "Stars"
export default file.particles({
  "maxParticles": 90,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.275,
      4.275,
      4.275
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
    2.6667,
    3.3333
  ],
  "speed": [
    0.1169,
    0.4677
  ],
  "size": [
    0.1125,
    0.1125
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
      0.8889,
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
      0.8889,
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
