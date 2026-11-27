import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2.0.0 "SparkEmitter"
export default file.particles({
  "maxParticles": 195,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.24,
      0.24,
      0.24
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 26,
      "cycles": 3,
      "interval": 0.025
    }
  ],
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.6667,
    1.1667
  ],
  "speed": [
    7.2,
    14.4
  ],
  "size": [
    0.72,
    0.72
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.5091,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -15.0,
    15.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "gravity": [
    0.0,
    12.96,
    -12.96
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Feather"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-2-0-0.effect"
  }
});
