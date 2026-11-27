import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.2 "CosmosSmoke_White"
export default file.particles({
  "maxParticles": 30,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.375,
      3.375,
      3.375
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.0667
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    2.0833,
    2.5833
  ],
  "speed": [
    0.0,
    0.0
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
        0.9706,
        0.9779,
        0.9965,
        0.1266
      ]
    ],
    [
      0.2031,
      [
        0.792,
        0.8418,
        0.9741,
        0.1339
      ]
    ],
    [
      0.5625,
      [
        0.3515,
        0.4807,
        0.9046,
        0.1572
      ]
    ],
    [
      0.6719,
      [
        0.243,
        0.3809,
        0.8812,
        0.1652
      ]
    ],
    [
      0.7857,
      [
        0.1529,
        0.2906,
        0.8572,
        0.1736
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.1636,
        0.8161,
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-0-0-2.effect"
  }
});
