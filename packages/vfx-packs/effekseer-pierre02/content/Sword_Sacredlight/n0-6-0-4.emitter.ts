import { file } from '@origingame/engine/author';
// Effekseer node 0.6.0.4 "FireVariation"
export default file.particles({
  "maxParticles": 65,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.7861,
      3.3241,
      0.001
    ]
  },
  "spread": 42.0333,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 4,
      "interval": 0.0417
    }
  ],
  "duration": 0.1417,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    5.7931,
    13.8421
  ],
  "speedOverLife": [
    [
      0.0,
      0.9016
    ],
    [
      1.0,
      1.0796
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
        1.0,
        0.8751,
        0.7519,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.8751,
        0.7519,
        0.0627
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8751,
        0.7519,
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Sacredlight/n0-6-0-4.effect"
  }
});
