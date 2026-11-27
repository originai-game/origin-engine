import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0 "BackSmoke"
export default file.particles({
  "maxParticles": 23,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.25,
      0.001,
      1.25
    ]
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 0.03,
  "looping": false,
  "lifetime": [
    0.0167,
    1.5
  ],
  "speed": [
    2.6517,
    10.6066
  ],
  "size": [
    8.857,
    8.857
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1411
    ],
    [
      0.125,
      0.3284
    ],
    [
      0.25,
      0.4928
    ],
    [
      0.375,
      0.6344
    ],
    [
      0.5,
      0.7532
    ],
    [
      0.625,
      0.8491
    ],
    [
      0.75,
      0.9222
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.011,
      [
        1.0,
        1.0,
        1.0,
        0.0063
      ]
    ],
    [
      0.1719,
      [
        1.0,
        1.0,
        1.0,
        0.0822
      ]
    ],
    [
      0.3297,
      [
        1.0,
        1.0,
        1.0,
        0.1276
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -300.0,
    300.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor_Single/n0-0-1-0.effect"
  }
});
