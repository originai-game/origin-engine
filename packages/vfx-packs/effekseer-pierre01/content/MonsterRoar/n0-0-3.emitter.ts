import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3 "SpitVariation"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.3,
      0.3736,
      0.2011
    ]
  },
  "spread": 76.4858,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.0417
    }
  ],
  "duration": 0.0583,
  "looping": false,
  "lifetime": [
    0.3333,
    1.3333
  ],
  "speed": [
    3.0741,
    6.3
  ],
  "size": [
    2.0,
    2.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.7519,
        0.8751,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.7519,
        0.8751,
        1.0,
        0.1255
      ]
    ],
    [
      0.6,
      [
        0.7519,
        0.8751,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.8751,
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
    -120.0,
    120.0
  ],
  "gravity": [
    0.0,
    -7.2,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Splash"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/MonsterRoar/n0-0-3.effect"
  }
});
