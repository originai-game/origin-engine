import { file } from '@origingame/engine/author';
// Effekseer node 0.4.0.5 "FireVariation"
export default file.particles({
  "maxParticles": 49,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.5298,
      2.5298,
      0.001
    ]
  },
  "spread": 27.4858,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    12.7352,
    25.0115
  ],
  "speedOverLife": [
    [
      0.0,
      0.8603
    ],
    [
      1.0,
      1.0934
    ]
  ],
  "size": [
    10.4,
    10.4
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
        0.1115
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.2195,
        0.0478,
        0.212
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.2195,
        0.0478,
        0.3493
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2195,
        0.0478,
        0.502
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Ember/n0-4-0-5.effect"
  }
});
