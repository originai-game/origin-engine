import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1.0.3 "FireVariation"
export default file.particles({
  "maxParticles": 68,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.15,
      0.7349
    ]
  },
  "spread": 26.2719,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 27,
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
    4.6572,
    11.6121
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
    8.3138,
    13.8564
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
        0.1166,
        0.0104,
        0.0
      ]
    ],
    [
      0.0469,
      [
        1.0,
        0.1166,
        0.0104,
        0.0648
      ]
    ],
    [
      0.0781,
      [
        1.0,
        0.1166,
        0.0104,
        0.1305
      ]
    ],
    [
      0.1094,
      [
        1.0,
        0.1166,
        0.0104,
        0.2325
      ]
    ],
    [
      0.1389,
      [
        1.0,
        0.1166,
        0.0104,
        0.3765
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.1166,
        0.0104,
        0.3765
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
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
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-0-3.effect"
  }
});
