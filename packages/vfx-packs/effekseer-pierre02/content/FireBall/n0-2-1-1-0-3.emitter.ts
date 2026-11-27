import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1.1.0.3 "FireVariation"
export default file.particles({
  "maxParticles": 68,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.9,
      1.5,
      0.9
    ]
  },
  "spread": 18.1561,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 18
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5167,
    0.6833
  ],
  "speed": [
    45.9898,
    70.5051
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
    9.5,
    9.5
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
        0.1166,
        0.0104,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.1166,
        0.0104,
        0.0482
      ]
    ],
    [
      0.2778,
      [
        1.0,
        0.1166,
        0.0104,
        0.1088
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
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
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-3.effect"
  }
});
