import { file } from '@origingame/engine/author';
// Effekseer node 0.8.0.0.0.4.0.1 "Light"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.6589,
      0.5271
    ]
  },
  "spread": 32.9053,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    1.5,
    1.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "speedOverLife": [
    [
      0.0,
      4.7561
    ],
    [
      0.1094,
      3.1314
    ],
    [
      0.2188,
      1.9366
    ],
    [
      0.2812,
      1.4781
    ],
    [
      0.3281,
      1.2412
    ],
    [
      0.4375,
      0.9869
    ],
    [
      0.4531,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.0,
    0.0
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
        0.7519,
        0.6388,
        0.0
      ]
    ],
    [
      0.1333,
      [
        1.0,
        0.7519,
        0.6388,
        0.0314
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.7519,
        0.6388,
        0.0314
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.6388,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.8671,
        0.8751,
        0.0
      ]
    ],
    [
      0.1333,
      [
        1.0,
        0.8671,
        0.8751,
        0.1569
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.8671,
        0.8751,
        0.1569
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8671,
        0.8751,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
