import { file } from '@origingame/engine/author';
// Effekseer node 0.1.5.0.4 "Light"
export default file.particles({
  "maxParticles": 60,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.9991,
      0.0428
    ]
  },
  "spread": 3.8234,
  "rate": 1200.0,
  "duration": 0.04,
  "looping": false,
  "lifetime": [
    1.0333,
    1.3
  ],
  "speed": [
    13.8468,
    24.9696
  ],
  "speedOverLife": [
    [
      0.0,
      2.2954
    ],
    [
      0.2188,
      1.5935
    ],
    [
      0.4219,
      1.061
    ],
    [
      0.6562,
      0.6051
    ],
    [
      0.9062,
      0.3059
    ],
    [
      0.9219,
      0.273
    ],
    [
      0.9375,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    12.0,
    12.0
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
        0.2845,
        0.0478,
        0.0
      ]
    ],
    [
      0.1714,
      [
        1.0,
        0.2845,
        0.0478,
        0.1569
      ]
    ],
    [
      0.5714,
      [
        1.0,
        0.2845,
        0.0478,
        0.1569
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2845,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.4423,
        0.1166,
        0.0
      ]
    ],
    [
      0.1714,
      [
        1.0,
        0.4423,
        0.1166,
        0.2196
      ]
    ],
    [
      0.5714,
      [
        1.0,
        0.4423,
        0.1166,
        0.2196
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.4423,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Particle_Soft"
  },
  "blend": "additive"
});
