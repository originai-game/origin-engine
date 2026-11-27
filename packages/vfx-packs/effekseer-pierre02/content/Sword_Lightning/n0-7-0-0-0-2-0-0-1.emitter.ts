import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.0.2.0.0.1 "CloudShadow_Variation2"
export default file.particles({
  "maxParticles": 75,
  "shape": {
    "kind": "box",
    "halfExtents": [
      9.0,
      6.0,
      9.0
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    5.6667,
    7.6667
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
        0.0199,
        0.029,
        0.0478,
        0.0
      ]
    ],
    [
      0.03,
      [
        0.0199,
        0.029,
        0.0478,
        0.3765
      ]
    ],
    [
      0.85,
      [
        0.0199,
        0.029,
        0.0478,
        0.3765
      ]
    ],
    [
      1.0,
      [
        0.0199,
        0.029,
        0.0478,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -12.0,
    12.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-0-0-2-0-0-1.effect"
  }
});
