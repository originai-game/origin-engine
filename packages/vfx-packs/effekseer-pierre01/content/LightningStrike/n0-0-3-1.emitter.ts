import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.1 "Debris_Variation2"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      2.0,
      4.0
    ]
  },
  "spread": 81.9505,
  "rate": 375.0,
  "duration": 0.08,
  "looping": false,
  "lifetime": [
    0.6667,
    1.3333
  ],
  "speed": [
    42.8486,
    59.6992
  ],
  "size": [
    0.7,
    2.3
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.017,
        0.017,
        0.017,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.017,
        0.017,
        0.017,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.017,
        0.017,
        0.017,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -300.0,
    300.0
  ],
  "gravity": [
    0.0,
    -36.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Glass"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-1.effect"
  }
});
