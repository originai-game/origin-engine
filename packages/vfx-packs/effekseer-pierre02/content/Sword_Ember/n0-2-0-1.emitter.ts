import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.1 "SmokeVariation"
export default file.particles({
  "maxParticles": 49,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2744,
      3.5672,
      0.001
    ]
  },
  "spread": 98.0495,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 3,
      "interval": 0.0333
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.75,
    1.5833
  ],
  "speed": [
    2.1377,
    10.6887
  ],
  "speedOverLife": [
    [
      0.0,
      0.6069
    ],
    [
      0.1562,
      1.0531
    ],
    [
      0.3125,
      1.3517
    ],
    [
      0.6094,
      1.4749
    ],
    [
      0.7656,
      1.3061
    ],
    [
      0.7812,
      1.0082
    ],
    [
      0.7969,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    19.455,
    32.425
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6168
    ],
    [
      0.25,
      0.6418
    ],
    [
      0.5,
      0.714
    ],
    [
      0.75,
      0.8334
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
      0.1429,
      [
        0.0781,
        0.0697,
        0.0619,
        0.0392
      ]
    ],
    [
      0.4286,
      [
        0.0781,
        0.0697,
        0.0619,
        0.0784
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Ember/n0-2-0-1.effect"
  }
});
