import { file } from '@origingame/engine/author';
// Effekseer node 0.6.0.0.0 "Light"
export default file.particles({
  "maxParticles": 31,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.2667
    }
  ],
  "duration": 0.2833,
  "looping": false,
  "lifetime": [
    0.5167,
    0.6833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    16.0,
    16.0
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
      0.1667,
      [
        1.0,
        0.1636,
        0.0254,
        0.0627
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.1636,
        0.0254,
        0.1004
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
      0.1667,
      [
        1.0,
        0.2845,
        0.0781,
        0.1882
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2845,
        0.0781,
        0.3012
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
