import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.3 "CosmosSmoke_Green"
export default file.particles({
  "maxParticles": 45,
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
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
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
    6.75,
    6.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0041,
        0.1636,
        0.2195,
        0.0
      ]
    ],
    [
      0.1071,
      [
        0.0041,
        0.1636,
        0.2195,
        0.1255
      ]
    ],
    [
      0.7857,
      [
        0.0041,
        0.1636,
        0.2195,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.0041,
        0.1636,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0677,
        0.1636,
        0.3297,
        0.0
      ]
    ],
    [
      0.1071,
      [
        0.0677,
        0.1636,
        0.3297,
        0.3765
      ]
    ],
    [
      0.7857,
      [
        0.0677,
        0.1636,
        0.3297,
        0.3765
      ]
    ],
    [
      1.0,
      [
        0.0677,
        0.1636,
        0.3297,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-0-0-3.effect"
  }
});
