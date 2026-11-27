import { file } from '@origingame/engine/author';
// Effekseer node 0.11.1.1.0.3 "FireVariation"
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
  "spread": 44.9504,
  "rate": 10.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    10.8094,
    80.0153
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
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-1-1-0-3.effect"
  }
});
