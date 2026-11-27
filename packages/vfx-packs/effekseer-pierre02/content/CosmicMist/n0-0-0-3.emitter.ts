import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3 "CosmosSmoke_White"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.5,
      7.955,
      1.591
    ]
  },
  "spread": 26.5651,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 12,
      "interval": 0.0167
    }
  ],
  "duration": 0.2,
  "looping": false,
  "lifetime": [
    2.5833,
    3.0833
  ],
  "speed": [
    0.9546,
    2.8638
  ],
  "size": [
    13.5,
    13.5
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0294,
      [
        0.984,
        0.984,
        0.9961,
        0.0637
      ]
    ],
    [
      0.2031,
      [
        0.8587,
        0.8587,
        0.9644,
        0.0712
      ]
    ],
    [
      0.6094,
      [
        0.4873,
        0.4873,
        0.8564,
        0.0979
      ]
    ],
    [
      0.8235,
      [
        0.3211,
        0.3211,
        0.7961,
        0.1136
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        0.7519,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
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
      0.0294,
      [
        0.984,
        0.9888,
        0.998,
        0.0637
      ]
    ],
    [
      0.25,
      [
        0.8183,
        0.8706,
        0.976,
        0.0737
      ]
    ],
    [
      0.6094,
      [
        0.4873,
        0.6199,
        0.9244,
        0.0979
      ]
    ],
    [
      0.8235,
      [
        0.3211,
        0.4815,
        0.8916,
        0.1136
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.3889,
        0.8671,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -18.0,
    18.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/CosmicMist/n0-0-0-3.effect"
  }
});
