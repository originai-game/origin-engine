import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.0.1.2.0.0 "FireVariation"
export default file.particles({
  "maxParticles": 89,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.3,
      0.5,
      2.3
    ]
  },
  "spread": 20.6627,
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
    0.5167,
    0.6833
  ],
  "speed": [
    16.6962,
    26.6953
  ],
  "speedOverLife": [
    [
      0.0,
      1.3376
    ],
    [
      0.5156,
      0.9713
    ],
    [
      1.0,
      0.7377
    ]
  ],
  "size": [
    3.9056,
    3.9056
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
      1.25
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
      0.0938,
      [
        0.2195,
        0.0055,
        0.0104,
        0.14
      ]
    ],
    [
      0.125,
      [
        0.2195,
        0.0055,
        0.0104,
        0.2033
      ]
    ],
    [
      0.1667,
      [
        0.2195,
        0.0055,
        0.0104,
        0.3124
      ]
    ],
    [
      0.2188,
      [
        0.2195,
        0.0055,
        0.0104,
        0.4712
      ]
    ],
    [
      0.2778,
      [
        0.2195,
        0.0055,
        0.0104,
        0.7052
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
    -6.0,
    6.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/BloodLance/n0-2-0-0-0-1-2-0-0.effect"
  }
});
