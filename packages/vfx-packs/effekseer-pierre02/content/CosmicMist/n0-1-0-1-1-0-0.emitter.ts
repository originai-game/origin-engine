import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.1.1.0.0 "FireVariation"
export default file.particles({
  "maxParticles": 105,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.7594,
      1.2656,
      0.7594
    ]
  },
  "spread": 20.6627,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 42
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5167,
    0.6833
  ],
  "speed": [
    34.2667,
    54.7884
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
    8.0156,
    8.0156
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
      0.1667,
      [
        0.0478,
        0.2845,
        0.2195,
        0.0723
      ]
    ],
    [
      0.2778,
      [
        0.0478,
        0.2845,
        0.2195,
        0.1631
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
    "$ogRef": "base/vfx/effekseer-pierre02/CosmicMist/n0-1-0-1-1-0-0.effect"
  }
});
