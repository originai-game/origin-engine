import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.12 "Fire_Variation3"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5,
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
    60.0,
    60.0
  ],
  "size": [
    77.0,
    77.0
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
        0.8751,
        0.7519,
        1.0,
        0.0
      ]
    ],
    [
      0.1,
      [
        0.7564,
        0.5991,
        0.9636,
        0.0427
      ]
    ],
    [
      0.2812,
      [
        0.5836,
        0.3906,
        0.9061,
        0.0357
      ]
    ],
    [
      0.5,
      [
        0.4303,
        0.225,
        0.8487,
        0.0285
      ]
    ],
    [
      0.7344,
      [
        0.3133,
        0.1168,
        0.7986,
        0.0117
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
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
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Fire"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-12.effect"
  }
});
