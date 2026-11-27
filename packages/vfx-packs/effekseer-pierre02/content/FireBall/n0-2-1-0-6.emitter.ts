import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1.0.6 "SmokeVariation"
export default file.particles({
  "maxParticles": 135,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.2,
      3.2,
      3.2
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 27,
      "cycles": 2,
      "interval": 0.0833
    }
  ],
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    0.9167,
    1.5833
  ],
  "speed": [
    0.864,
    1.056
  ],
  "size": [
    17.16,
    20.28
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5128
    ],
    [
      0.25,
      0.5445
    ],
    [
      0.5,
      0.6362
    ],
    [
      0.75,
      0.7881
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
        0.0037,
        0.0023,
        0.0017,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.0037,
        0.0023,
        0.0017,
        0.0471
      ]
    ],
    [
      0.4,
      [
        0.0037,
        0.0023,
        0.0017,
        0.0706
      ]
    ],
    [
      1.0,
      [
        0.0037,
        0.0023,
        0.0017,
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
  "gravity": [
    0.0,
    8.64,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-6.effect"
  }
});
