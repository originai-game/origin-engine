import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.6.0.0 "FireVariation"
export default file.particles({
  "maxParticles": 120,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.375,
      2.25,
      3.375
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 12,
      "cycles": 2,
      "interval": 0.2
    }
  ],
  "duration": 0.2167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    21.6,
    21.6
  ],
  "size": [
    9.0,
    9.0
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
        0.0478,
        0.2845,
        0.2195,
        0.0
      ]
    ],
    [
      0.1875,
      [
        0.0478,
        0.2845,
        0.2195,
        0.0707
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.2845,
        0.2195,
        0.1673
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
  "gravity": [
    0.0,
    64.8,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-0-0-6-0-0.effect"
  }
});
