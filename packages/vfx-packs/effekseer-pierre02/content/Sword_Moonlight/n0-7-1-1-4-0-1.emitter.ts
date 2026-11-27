import { file } from '@origingame/engine/author';
// Effekseer node 0.7.1.1.4.0.1 "Light"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.1049,
      0.4347,
      0.001
    ]
  },
  "spread": 25.8947,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 21,
      "interval": 0.0417
    }
  ],
  "duration": 0.85,
  "looping": false,
  "lifetime": [
    1.3333,
    1.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "speedOverLife": [
    [
      0.0,
      0.6958
    ],
    [
      0.1406,
      1.2159
    ],
    [
      0.2656,
      1.5302
    ],
    [
      0.4062,
      1.6998
    ],
    [
      0.5312,
      1.687
    ],
    [
      0.6719,
      1.4885
    ],
    [
      0.7031,
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
        0.0478,
        0.2195,
        0.1636,
        0.0
      ]
    ],
    [
      0.15,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0314
      ]
    ],
    [
      0.625,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0314
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2195,
        0.1636,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.3587,
        0.2845,
        0.0
      ]
    ],
    [
      0.15,
      [
        0.0478,
        0.3587,
        0.2845,
        0.1569
      ]
    ],
    [
      0.625,
      [
        0.0478,
        0.3587,
        0.2845,
        0.1569
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.3587,
        0.2845,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
