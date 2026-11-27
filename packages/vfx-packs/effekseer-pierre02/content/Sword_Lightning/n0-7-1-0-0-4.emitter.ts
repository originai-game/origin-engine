import { file } from '@origingame/engine/author';
// Effekseer node 0.7.1.0.0.4 "Light"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.45,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 11,
      "interval": 0.05
    }
  ],
  "duration": 0.5167,
  "looping": false,
  "lifetime": [
    0.5167,
    0.6833
  ],
  "speed": [
    204.3005,
    209.7005
  ],
  "speedOverLife": [
    [
      0.0,
      0.6887
    ],
    [
      0.5,
      1.053
    ],
    [
      1.0,
      1.1063
    ]
  ],
  "size": [
    33.75,
    78.75
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8
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
        0.0494,
        0.0478,
        0.7519,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.0494,
        0.0478,
        0.7519,
        0.0
      ]
    ],
    [
      0.4167,
      [
        0.0494,
        0.0478,
        0.7519,
        0.0
      ]
    ],
    [
      1.0,
      [
        0.0494,
        0.0478,
        0.7519,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0494,
        0.1304,
        0.7519,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.0494,
        0.1304,
        0.7519,
        0.2008
      ]
    ],
    [
      0.2969,
      [
        0.0494,
        0.1304,
        0.7519,
        0.3018
      ]
    ],
    [
      0.4167,
      [
        0.0494,
        0.1304,
        0.7519,
        0.3514
      ]
    ],
    [
      1.0,
      [
        0.0494,
        0.1304,
        0.7519,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Star"
  },
  "blend": "additive"
});
