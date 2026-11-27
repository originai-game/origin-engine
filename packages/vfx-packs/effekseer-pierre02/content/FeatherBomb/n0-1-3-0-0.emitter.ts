import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3.0.0 "FireVariation"
export default file.particles({
  "maxParticles": 65,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.675,
      1.125,
      0.675
    ]
  },
  "spread": 20.6627,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 26
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5167,
    0.6833
  ],
  "speed": [
    30.4593,
    48.7008
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
    7.125,
    7.125
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0938,
      [
        1.0,
        1.0,
        1.0,
        0.1478
      ]
    ],
    [
      0.125,
      [
        1.0,
        1.0,
        1.0,
        0.2146
      ]
    ],
    [
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.3298
      ]
    ],
    [
      0.2188,
      [
        1.0,
        1.0,
        1.0,
        0.4973
      ]
    ],
    [
      0.2778,
      [
        1.0,
        1.0,
        1.0,
        0.7443
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
    -6.0,
    6.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-0.effect"
  }
});
