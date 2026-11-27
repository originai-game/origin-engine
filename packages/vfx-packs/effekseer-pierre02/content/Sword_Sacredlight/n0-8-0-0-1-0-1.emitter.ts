import { file } from '@origingame/engine/author';
// Effekseer node 0.8.0.0.1.0.1 "FireVariation"
export default file.particles({
  "maxParticles": 285,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5063,
      1.6875,
      0.5063
    ]
  },
  "spread": 44.9504,
  "rate": 8.5714,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    8.107,
    60.0115
  ],
  "speedOverLife": [
    [
      0.0,
      0.7443
    ],
    [
      1.0,
      1.2053
    ]
  ],
  "size": [
    10.125,
    10.125
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Sacredlight/n0-8-0-0-1-0-1.effect"
  }
});
