import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3.0.0 "SmokeVariation"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.9112,
      0.5893
    ]
  },
  "spread": 34.7562,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 20
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    1.0,
    2.0
  ],
  "speed": [
    1.538,
    6.88
  ],
  "speedOverLife": [
    [
      0.0,
      2.0737
    ],
    [
      0.2656,
      1.3816
    ],
    [
      0.5156,
      0.8767
    ],
    [
      0.7656,
      0.529
    ],
    [
      1.0,
      0.3504
    ]
  ],
  "size": [
    6.1069,
    10.1781
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2456
    ],
    [
      0.25,
      0.2943
    ],
    [
      0.375,
      0.3537
    ],
    [
      0.5,
      0.4363
    ],
    [
      0.625,
      0.5422
    ],
    [
      0.75,
      0.6715
    ],
    [
      0.875,
      0.8241
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
        0.0781,
        0.0697,
        0.0619,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.0781,
        0.0697,
        0.0619,
        0.1882
      ]
    ],
    [
      1.0,
      [
        0.0781,
        0.0697,
        0.0619,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -180.0,
    180.0
  ],
  "gravity": [
    0.0,
    0.72,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/Flame/n0-1-3-0-0.effect"
  }
});
