import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1.1.0.1 "FireVariation"
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
  "spread": 20.6627,
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
    40.6124,
    64.9344
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
      0.0938,
      [
        1.0,
        0.1166,
        0.0104,
        0.0648
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.1166,
        0.0104,
        0.1445
      ]
    ],
    [
      0.2188,
      [
        1.0,
        0.1166,
        0.0104,
        0.218
      ]
    ],
    [
      0.2778,
      [
        1.0,
        0.1166,
        0.0104,
        0.3263
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
    "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-1.effect"
  }
});
