import { file } from '@origingame/engine/author';
// Effekseer node 0.5.1.0.1 "FireVariation"
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.3414,
      0.2889,
      0.001
    ]
  },
  "spread": 58.1694,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 21,
      "interval": 0.0417
    }
  ],
  "duration": 0.85,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    2.1377,
    12.2804
  ],
  "speedOverLife": [
    [
      0.0,
      0.5292
    ],
    [
      0.5,
      1.0446
    ],
    [
      1.0,
      1.2965
    ]
  ],
  "size": [
    8.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.25
    ],
    [
      1.0,
      2.0
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
        0.2845,
        0.2195,
        0.0
      ]
    ],
    [
      0.1094,
      [
        0.0478,
        0.2845,
        0.2195,
        0.1115
      ]
    ],
    [
      0.1875,
      [
        0.0478,
        0.2845,
        0.2195,
        0.212
      ]
    ],
    [
      0.2656,
      [
        0.0478,
        0.2845,
        0.2195,
        0.3493
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.2845,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2845,
        0.2195,
        0.0
      ]
    ]
  ],
  "rotation": [
    -6.0,
    6.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-5-1-0-1.effect"
  }
});
