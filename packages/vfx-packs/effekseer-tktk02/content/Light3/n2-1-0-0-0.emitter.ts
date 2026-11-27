import { file } from '@origingame/engine/author';
// Effekseer node 2.1.0.0.0 "particleMain"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      2.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 200
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.3929,
    5.0357
  ],
  "speedOverLife": [
    [
      0.0,
      3.2307
    ],
    [
      0.25,
      1.7283
    ],
    [
      0.375,
      1.1437
    ],
    [
      0.5,
      0.6794
    ],
    [
      0.625,
      0.3353
    ],
    [
      0.75,
      0.1115
    ],
    [
      0.875,
      0.0079
    ],
    [
      1.0,
      0.0223
    ]
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
