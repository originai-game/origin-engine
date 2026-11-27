import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.0.1 "FireVariation"
export default file.particles({
  "maxParticles": 180,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.6,
      1.0,
      0.6
    ]
  },
  "spread": 20.6627,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 72
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5167,
    0.6833
  ],
  "speed": [
    27.0749,
    43.2896
  ],
  "speedOverLife": [
    [
      0.0,
      1.3376
    ],
    [
      0.5156,
      0.9713
    ],
    [
      1.0,
      0.7377
    ]
  ],
  "size": [
    6.3333,
    6.3333
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
        0.2195,
        0.0055,
        0.0104,
        0.0
      ]
    ],
    [
      0.0938,
      [
        0.2195,
        0.0055,
        0.0104,
        0.1296
      ]
    ],
    [
      0.1667,
      [
        0.2195,
        0.0055,
        0.0104,
        0.2891
      ]
    ],
    [
      0.2188,
      [
        0.2195,
        0.0055,
        0.0104,
        0.436
      ]
    ],
    [
      0.2778,
      [
        0.2195,
        0.0055,
        0.0104,
        0.6525
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0055,
        0.0104,
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
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/LossOfBlood/n0-1-1-0-1.effect"
  }
});
