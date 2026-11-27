import { file } from '@origingame/engine/author';
// Effekseer node 0.6.1.2.6 "Fire_Variation1"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 2,
      "interval": 0.0333
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.25,
    0.75
  ],
  "speed": [
    240.0,
    240.0
  ],
  "size": [
    308.0,
    308.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4156
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.2078
    ],
    [
      1.0,
      0.987
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
        1.0,
        0.9786,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        0.9663,
        1.0,
        0.0427
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.9264,
        1.0,
        0.0285
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8913,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -5.0,
    5.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Storm/n0-6-1-2-6.effect"
  }
});
