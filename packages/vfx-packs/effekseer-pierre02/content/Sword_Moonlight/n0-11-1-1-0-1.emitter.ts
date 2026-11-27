import { file } from '@origingame/engine/author';
// Effekseer node 0.11.1.1.0.1 "FireVariation"
export default file.particles({
  "maxParticles": 240,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.675,
      2.25,
      0.675
    ]
  },
  "spread": 120.5089,
  "rate": 10.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    8.864,
    67.9306
  ],
  "speedOverLife": [
    [
      0.0,
      0.6149
    ],
    [
      0.5156,
      1.0303
    ],
    [
      1.0,
      1.3091
    ]
  ],
  "size": [
    13.5,
    13.5
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
        0.0884
      ]
    ],
    [
      0.3333,
      [
        0.0478,
        0.2845,
        0.2195,
        0.2092
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-1-1-0-1.effect"
  }
});
