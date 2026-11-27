import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1.0.0 "Light"
export default file.particles({
  "maxParticles": 68,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.15,
      0.7349
    ]
  },
  "spread": 26.2719,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 27,
      "cycles": 2,
      "interval": 0.025
    }
  ],
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.7,
    1.3
  ],
  "speed": [
    4.6572,
    11.6121
  ],
  "speedOverLife": [
    [
      0.0,
      1.9319
    ],
    [
      0.25,
      1.3142
    ],
    [
      0.5,
      0.8743
    ],
    [
      0.75,
      0.6232
    ],
    [
      1.0,
      0.5603
    ]
  ],
  "size": [
    19.2,
    19.2
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
      0.05,
      [
        1.0,
        0.1636,
        0.0254,
        0.0627
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.1636,
        0.0254,
        0.0627
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
      0.05,
      [
        1.0,
        0.2845,
        0.0781,
        0.1255
      ]
    ],
    [
      0.25,
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
