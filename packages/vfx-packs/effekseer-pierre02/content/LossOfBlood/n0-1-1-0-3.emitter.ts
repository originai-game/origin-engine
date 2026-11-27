import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.0.3 "FireVariation"
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
  "spread": 18.1561,
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
    30.6598,
    47.0034
  ],
  "speedOverLife": [
    [
      0.0,
      1.3054
    ],
    [
      0.5156,
      0.9741
    ],
    [
      1.0,
      0.7628
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
        0.0432
      ]
    ],
    [
      0.1667,
      [
        0.2195,
        0.0055,
        0.0104,
        0.0964
      ]
    ],
    [
      0.2778,
      [
        0.2195,
        0.0055,
        0.0104,
        0.2175
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
    "$ogRef": "base/vfx/effekseer-pierre02/LossOfBlood/n0-1-1-0-3.effect"
  }
});
