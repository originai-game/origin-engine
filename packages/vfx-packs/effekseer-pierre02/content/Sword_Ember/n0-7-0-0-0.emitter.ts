import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.0 "FireVariation"
export default file.particles({
  "maxParticles": 60,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.826,
      1.3147
    ]
  },
  "spread": 26.0675,
  "rate": 300.0,
  "duration": 0.16,
  "looping": false,
  "lifetime": [
    0.2,
    0.3
  ],
  "speed": [
    11.475,
    30.2548
  ],
  "speedOverLife": [
    [
      0.0,
      1.3481
    ],
    [
      1.0,
      0.7036
    ]
  ],
  "size": [
    11.6913,
    19.4856
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
      0.8115,
      0.5843
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
        0.0835
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.2195,
        0.0478,
        0.174
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.2195,
        0.0478,
        0.3171
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
    -9.0,
    9.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Ember/n0-7-0-0-0.effect"
  }
});
