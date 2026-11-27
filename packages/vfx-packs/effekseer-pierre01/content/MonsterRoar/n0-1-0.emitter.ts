import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "Sigh"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "point"
  },
  "spread": 64.7606,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 18,
      "interval": 0.0833
    }
  ],
  "duration": 1.4333,
  "looping": false,
  "lifetime": [
    0.25,
    0.75
  ],
  "speed": [
    5.6285,
    8.8182
  ],
  "size": [
    7.0,
    7.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5714
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.0314
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.0314
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
    "$ogRef": "base/vfx/effekseer-pierre01/MonsterRoar/n0-1-0.effect"
  }
});
