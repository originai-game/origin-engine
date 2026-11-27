import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.4 "CosmosSmoke_White"
export default file.particles({
  "maxParticles": 30,
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
      "cycles": 2,
      "interval": 0.0667
    }
  ],
  "duration": 0.0833,
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
        0.9761,
        0.982,
        0.9972,
        0.0946
      ]
    ],
    [
      0.2031,
      [
        0.792,
        0.8418,
        0.9741,
        0.0983
      ]
    ],
    [
      0.5781,
      [
        0.3348,
        0.4658,
        0.9012,
        0.1105
      ]
    ],
    [
      0.7031,
      [
        0.2158,
        0.3546,
        0.8745,
        0.1151
      ]
    ],
    [
      0.8235,
      [
        0.1284,
        0.264,
        0.8495,
        0.1195
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
    -18.0,
    18.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/CosmicMist/n0-0-0-4.effect"
  }
});
