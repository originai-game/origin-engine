import { file } from '@origingame/engine/author';
// Effekseer node 0.5.0.4 "FireVariation"
export default file.particles({
  "maxParticles": 33,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2744,
      3.5672,
      0.001
    ]
  },
  "spread": 98.0495,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 2,
      "interval": 0.0333
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    2.1377,
    10.6887
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
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.1875,
      [
        0.0478,
        0.2195,
        1.0,
        0.106
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.2195,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.2195,
        1.0,
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-5-0-4.effect"
  }
});
