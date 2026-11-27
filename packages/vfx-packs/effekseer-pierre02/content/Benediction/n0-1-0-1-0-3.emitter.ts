import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.1.0.3 "FireVariation"
export default file.particles({
  "maxParticles": 105,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1265,
      0.62
    ]
  },
  "spread": 26.2719,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 42,
      "cycles": 2,
      "interval": 0.025
    }
  ],
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.55,
    0.65
  ],
  "speed": [
    3.9295,
    9.7977
  ],
  "speedOverLife": [
    [
      0.0,
      1.5412
    ],
    [
      0.5156,
      0.9492
    ],
    [
      1.0,
      0.5999
    ]
  ],
  "size": [
    7.0148,
    11.6913
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5774
    ],
    [
      0.3281,
      0.6843
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.2,
      0.9798
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.8751,
        0.7519,
        0.5356,
        0.0
      ]
    ],
    [
      0.0781,
      [
        0.8751,
        0.7519,
        0.5356,
        0.0353
      ]
    ],
    [
      0.1389,
      [
        0.8751,
        0.7519,
        0.5356,
        0.102
      ]
    ],
    [
      0.3333,
      [
        0.8751,
        0.7519,
        0.5356,
        0.102
      ]
    ],
    [
      1.0,
      [
        0.8751,
        0.7519,
        0.5356,
        0.0
      ]
    ]
  ],
  "rotation": [
    -15.0,
    15.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-0-3.effect"
  }
});
