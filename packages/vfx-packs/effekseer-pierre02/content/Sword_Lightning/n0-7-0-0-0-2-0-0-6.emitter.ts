import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.0.2.0.0.6 "CloudLighting_Variation3"
export default file.particles({
  "maxParticles": 45,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.5,
      6.0,
      1.5
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 12,
      "cycles": 3,
      "interval": 0.25
    }
  ],
  "duration": 0.5167,
  "looping": false,
  "lifetime": [
    2.5,
    4.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    24.0,
    24.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0658,
        0.0915,
        0.259,
        0.0
      ]
    ],
    [
      0.1429,
      [
        0.0658,
        0.0915,
        0.259,
        0.1255
      ]
    ],
    [
      0.7143,
      [
        0.0658,
        0.0915,
        0.259,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.0658,
        0.0915,
        0.259,
        0.0
      ]
    ]
  ],
  "rotation": [
    150.0,
    210.0
  ],
  "angularVelocity": [
    -6.0,
    6.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke_Lighting"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-0-0-2-0-0-6.effect"
  }
});
