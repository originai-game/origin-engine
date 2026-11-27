import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "SmokeVariation"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "point"
  },
  "spread": 8.0495,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 5,
      "interval": 0.0667
    }
  ],
  "duration": 0.2833,
  "looping": false,
  "lifetime": [
    0.5,
    1.5
  ],
  "speed": [
    6.0597,
    24.015
  ],
  "size": [
    5.2725,
    8.7875
  ],
  "sizeOverLife": [
    [
      0.0,
      0.569
    ],
    [
      0.25,
      0.6287
    ],
    [
      0.5,
      0.7205
    ],
    [
      0.75,
      0.8442
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
        0.7519,
        0.7519,
        0.8122,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.7519,
        0.7519,
        0.8122,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.7519,
        0.8122,
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
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/MissileBoost/n0-2.effect"
  }
});
