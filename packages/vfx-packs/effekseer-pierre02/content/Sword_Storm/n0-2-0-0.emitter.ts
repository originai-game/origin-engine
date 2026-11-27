import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0 "SmokeVariation"
export default file.particles({
  "maxParticles": 65,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 2,
      "interval": 0.025
    }
  ],
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.75,
    1.5833
  ],
  "speed": [
    4.8,
    14.4
  ],
  "size": [
    14.352,
    21.528
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4459
    ],
    [
      0.25,
      0.482
    ],
    [
      0.5,
      0.5864
    ],
    [
      0.75,
      0.7591
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
        0.9743,
        0.9871,
        1.0,
        0.0
      ]
    ],
    [
      0.1429,
      [
        0.9743,
        0.9871,
        1.0,
        0.0261
      ]
    ],
    [
      0.4286,
      [
        0.9743,
        0.9871,
        1.0,
        0.0523
      ]
    ],
    [
      1.0,
      [
        0.9743,
        0.9871,
        1.0,
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
    5.76,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Storm/n0-2-0-0.effect"
  }
});
