import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2.4 "FireVariation"
export default file.particles({
  "maxParticles": 65,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1125,
      0.5511
    ]
  },
  "spread": 26.2719,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 26,
      "cycles": 2,
      "interval": 0.025
    }
  ],
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.55,
    0.65
  ],
  "speed": [
    3.4929,
    8.7091
  ],
  "speedOverLife": [
    [
      0.0,
      1.5412
    ],
    [
      0.5156,
      0.9492
    ],
    [
      1.0,
      0.5999
    ]
  ],
  "size": [
    6.2354,
    10.3923
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5774
    ],
    [
      0.3281,
      0.6843
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
      0.2,
      0.9798
    ]
  },
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
      0.0781,
      [
        1.0,
        1.0,
        1.0,
        0.087
      ]
    ],
    [
      0.1094,
      [
        1.0,
        1.0,
        1.0,
        0.155
      ]
    ],
    [
      0.1389,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.251
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-2-4.effect"
  }
});
