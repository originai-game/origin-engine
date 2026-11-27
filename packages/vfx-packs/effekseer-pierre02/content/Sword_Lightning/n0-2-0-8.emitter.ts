import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.8 "EmberVariation"
export default file.particles({
  "maxParticles": 33,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      2.0,
      0.001
    ]
  },
  "spread": 135.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 3,
      "interval": 0.35
    }
  ],
  "duration": 0.7167,
  "looping": false,
  "lifetime": [
    0.05,
    0.65
  ],
  "speed": [
    6.7882,
    15.1789
  ],
  "size": [
    4.0,
    4.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.2188,
      0.4297
    ],
    [
      0.4531,
      0.6358
    ],
    [
      0.7031,
      0.8189
    ],
    [
      1.0,
      1.0
    ]
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
        0.0478,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.1406,
      [
        0.0478,
        0.2195,
        1.0,
        0.1592
      ]
    ],
    [
      0.2857,
      [
        0.0478,
        0.2195,
        1.0,
        0.2689
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -30.0,
    30.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "gravity": [
    -3.074,
    5.5498,
    -9.6156
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Splash"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-2-0-8.effect"
  }
});
