import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2 "CosmosSmoke_Green"
export default file.particles({
  "maxParticles": 45,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.5,
      4.5,
      4.5
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
    2.5833,
    3.0833
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    9.0,
    9.0
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
      0.0882,
      [
        0.0041,
        0.1636,
        0.2195,
        0.0314
      ]
    ],
    [
      0.8235,
      [
        0.0041,
        0.1636,
        0.2195,
        0.0314
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
      0.0882,
      [
        0.0677,
        0.1636,
        0.3297,
        0.2196
      ]
    ],
    [
      0.8235,
      [
        0.0677,
        0.1636,
        0.3297,
        0.2196
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
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/CosmicMist/n0-0-0-2.effect"
  }
});
