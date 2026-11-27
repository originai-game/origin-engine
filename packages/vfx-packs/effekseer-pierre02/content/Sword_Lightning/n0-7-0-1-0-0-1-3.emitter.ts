import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.3 "Debris_Variation4"
export default file.particles({
  "maxParticles": 42,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      2.0,
      4.0
    ]
  },
  "spread": 81.9505,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 3,
      "interval": 0.0917
    }
  ],
  "duration": 0.2,
  "looping": false,
  "lifetime": [
    0.6667,
    1.3333
  ],
  "speed": [
    29.994,
    41.7895
  ],
  "size": [
    0.84,
    1.96
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
    -25.2,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Glass"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-1-0-0-1-3.effect"
  }
});
