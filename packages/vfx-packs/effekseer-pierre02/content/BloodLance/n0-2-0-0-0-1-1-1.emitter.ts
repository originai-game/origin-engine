import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.0.1.1.1 "FireVariation"
export default file.particles({
  "maxParticles": 89,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      0.001,
      2.25
    ]
  },
  "spread": 23.3177,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 40,
      "interval": 0.0267
    }
  ],
  "duration": 1.0567,
  "looping": false,
  "lifetime": [
    0.55,
    0.65
  ],
  "speed": [
    2.0772,
    3.3263
  ],
  "speedOverLife": [
    [
      0.0,
      1.5121
    ],
    [
      0.5156,
      0.952
    ],
    [
      1.0,
      0.6212
    ]
  ],
  "size": [
    3.4179,
    5.6965
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
      0.0,
      1.0
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
        0.1296
      ]
    ],
    [
      0.0781,
      [
        0.2195,
        0.0055,
        0.0104,
        0.261
      ]
    ],
    [
      0.1094,
      [
        0.2195,
        0.0055,
        0.0104,
        0.4651
      ]
    ],
    [
      0.125,
      [
        0.2195,
        0.0055,
        0.0104,
        0.6042
      ]
    ],
    [
      0.1389,
      [
        0.2195,
        0.0055,
        0.0104,
        0.7529
      ]
    ],
    [
      0.3333,
      [
        0.2195,
        0.0055,
        0.0104,
        0.7529
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-1-1.effect"
  }
});
