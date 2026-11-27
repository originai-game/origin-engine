import { file } from '@origingame/engine/author';
// Effekseer node 2.1.0.0.1 "particleGlow"
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
    2.5,
    2.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.4723,
        1.0,
        0.007,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.4423,
        1.0,
        0.0055,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.3587,
        1.0,
        0.0023,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7445,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.6707,
        1.0,
        0.032,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.6354,
        1.0,
        0.0254,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
