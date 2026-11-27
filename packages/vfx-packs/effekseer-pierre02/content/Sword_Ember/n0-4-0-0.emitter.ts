import { file } from '@origingame/engine/author';
// Effekseer node 0.4.0.0 "Light"
export default file.particles({
  "maxParticles": 49,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.7496,
      3.4983,
      0.001
    ]
  },
  "spread": 110.4393,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6667,
    0.8333
  ],
  "speed": [
    2.5369,
    15.5974
  ],
  "speedOverLife": [
    [
      0.0,
      0.4816
    ],
    [
      0.25,
      0.851
    ],
    [
      0.5,
      1.0909
    ],
    [
      0.75,
      1.1944
    ],
    [
      1.0,
      1.1632
    ]
  ],
  "size": [
    41.6,
    41.6
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
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
        0.0157
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.1636,
        0.0254,
        0.0157
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
        0.1412
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2845,
        0.0781,
        0.1412
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
