import { file } from '@origingame/engine/author';
// Effekseer node 0.7.6.0.3 "FireVariation"
export default file.particles({
  "maxParticles": 60,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 6,
      "interval": 0.4
    }
  ],
  "duration": 2.0167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    4.05,
    4.05
  ],
  "size": [
    4.5,
    4.5
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
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.1094,
      [
        1.0,
        0.2195,
        0.0478,
        0.1132
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.2195,
        0.0478,
        0.2154
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.2195,
        0.0478,
        0.3548
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2195,
        0.0478,
        0.5098
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Ember/n0-7-6-0-3.effect"
  }
});
