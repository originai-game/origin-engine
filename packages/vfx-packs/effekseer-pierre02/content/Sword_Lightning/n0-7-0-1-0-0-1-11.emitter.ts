import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.11 "Fire_Variation3"
export default file.particles({
  "maxParticles": 42,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
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
    0.25,
    0.75
  ],
  "speed": [
    42.0,
    42.0
  ],
  "size": [
    53.9,
    53.9
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
        0.5935,
        0.5991,
        0.9636,
        0.0427
      ]
    ],
    [
      0.2812,
      [
        0.3873,
        0.3906,
        0.9061,
        0.0357
      ]
    ],
    [
      0.5,
      [
        0.2234,
        0.225,
        0.8487,
        0.0285
      ]
    ],
    [
      0.7344,
      [
        0.1163,
        0.1168,
        0.7986,
        0.0117
      ]
    ],
    [
      1.0,
      [
        0.0478,
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
        0.8355,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.5935,
        0.7169,
        0.9648,
        0.0427
      ]
    ],
    [
      0.2812,
      [
        0.3873,
        0.5452,
        0.9091,
        0.0357
      ]
    ],
    [
      0.5,
      [
        0.2234,
        0.3943,
        0.8534,
        0.0285
      ]
    ],
    [
      0.7344,
      [
        0.1163,
        0.2804,
        0.8047,
        0.0117
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.1905,
        0.7593,
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-1-0-0-1-11.effect"
  }
});
