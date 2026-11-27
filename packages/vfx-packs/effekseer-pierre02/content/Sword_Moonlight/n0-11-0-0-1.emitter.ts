import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.1 "CosmosSmoke_White"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.375,
      4.773,
      0.001
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
    2.0833,
    2.5833
  ],
  "speed": [
    0.9546,
    2.8638
  ],
  "size": [
    9.0,
    9.0
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
      0.0357,
      [
        0.9804,
        0.9804,
        0.9952,
        0.0958
      ]
    ],
    [
      0.2031,
      [
        0.8587,
        0.8587,
        0.9644,
        0.1068
      ]
    ],
    [
      0.5938,
      [
        0.5009,
        0.5009,
        0.8609,
        0.1451
      ]
    ],
    [
      0.7857,
      [
        0.3472,
        0.3472,
        0.8064,
        0.1663
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
      0.0357,
      [
        0.9804,
        0.9862,
        0.9975,
        0.0958
      ]
    ],
    [
      0.25,
      [
        0.8183,
        0.8706,
        0.976,
        0.1106
      ]
    ],
    [
      0.5938,
      [
        0.5009,
        0.6308,
        0.9268,
        0.1451
      ]
    ],
    [
      0.7857,
      [
        0.3472,
        0.5042,
        0.8972,
        0.1663
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
    -120.0,
    120.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-0-0-1.effect"
  }
});
