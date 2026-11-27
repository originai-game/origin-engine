import { file } from '@origingame/engine/author';
// Effekseer node 0.3.0.4 "FireVariation"
export default file.particles({
  "maxParticles": 33,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.2744,
      2.7617,
      0.001
    ]
  },
  "spread": 32.2691,
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
    8.3053,
    15.1302
  ],
  "speedOverLife": [
    [
      0.0,
      0.8435
    ],
    [
      1.0,
      1.1071
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Sacredlight/n0-3-0-4.effect"
  }
});
