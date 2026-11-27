import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.4 "Light"
export default file.particles({
  "maxParticles": 60,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.9294,
      1.1576
    ]
  },
  "spread": 28.0725,
  "rate": 300.0,
  "duration": 0.16,
  "looping": false,
  "lifetime": [
    0.7,
    0.8
  ],
  "speed": [
    10.6727,
    29.0328
  ],
  "speedOverLife": [
    [
      0.0,
      2.3953
    ],
    [
      0.2031,
      1.6178
    ],
    [
      0.3906,
      1.0751
    ],
    [
      0.625,
      0.6548
    ],
    [
      0.875,
      0.5225
    ],
    [
      0.8906,
      0.2038
    ],
    [
      0.9062,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    18.0,
    18.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.625
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
        0.1636,
        0.0254,
        0.0
      ]
    ],
    [
      0.2667,
      [
        1.0,
        0.1636,
        0.0254,
        0.0314
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.1636,
        0.0254,
        0.0314
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1636,
        0.0254,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2845,
        0.0781,
        0.0
      ]
    ],
    [
      0.2667,
      [
        1.0,
        0.2845,
        0.0781,
        0.1255
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2845,
        0.0781,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2845,
        0.0781,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
