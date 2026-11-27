import { file } from '@origingame/engine/author';
// Effekseer node 0.7.2.4 "Fire_Variation1"
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
    54.0,
    54.0
  ],
  "size": [
    69.3,
    69.3
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
        0.7445,
        0.7519,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.5943,
        0.5991,
        0.9636,
        0.0427
      ]
    ],
    [
      0.2812,
      [
        0.3891,
        0.3906,
        0.9061,
        0.0357
      ]
    ],
    [
      0.5,
      [
        0.2257,
        0.225,
        0.8487,
        0.0285
      ]
    ],
    [
      0.7344,
      [
        0.1184,
        0.1168,
        0.7986,
        0.0117
      ]
    ],
    [
      1.0,
      [
        0.0494,
        0.0478,
        0.7519,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7445,
        0.7969,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.5943,
        0.6642,
        0.9636,
        0.0427
      ]
    ],
    [
      0.2812,
      [
        0.3891,
        0.4765,
        0.9061,
        0.0357
      ]
    ],
    [
      0.5,
      [
        0.2257,
        0.3176,
        0.8487,
        0.0285
      ]
    ],
    [
      0.7344,
      [
        0.1184,
        0.2035,
        0.7986,
        0.0117
      ]
    ],
    [
      1.0,
      [
        0.0494,
        0.1193,
        0.7519,
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-2-4.effect"
  }
});
