import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.2 "FireVariation"
export default file.particles({
  "maxParticles": 180,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1,
      0.4899
    ]
  },
  "spread": 26.2719,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 72,
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
    3.1048,
    7.7414
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
    5.5426,
    9.2376
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
        0.2195,
        0.0055,
        0.0104,
        0.0
      ]
    ],
    [
      0.0469,
      [
        0.2195,
        0.0055,
        0.0104,
        0.0864
      ]
    ],
    [
      0.0781,
      [
        0.2195,
        0.0055,
        0.0104,
        0.174
      ]
    ],
    [
      0.1094,
      [
        0.2195,
        0.0055,
        0.0104,
        0.3101
      ]
    ],
    [
      0.1389,
      [
        0.2195,
        0.0055,
        0.0104,
        0.502
      ]
    ],
    [
      0.3333,
      [
        0.2195,
        0.0055,
        0.0104,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0055,
        0.0104,
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
    "$ogRef": "base/vfx/effekseer-pierre02/LossOfBlood/n0-1-0-2.effect"
  }
});
