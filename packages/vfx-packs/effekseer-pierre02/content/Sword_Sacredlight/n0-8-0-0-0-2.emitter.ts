import { file } from '@origingame/engine/author';
// Effekseer node 0.8.0.0.0.2 "FireVariation"
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5848,
      0.6082
    ]
  },
  "spread": 28.3526,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.2,
    0.3
  ],
  "speed": [
    4.776,
    13.6218
  ],
  "speedOverLife": [
    [
      0.0,
      1.3336
    ],
    [
      1.0,
      0.7164
    ]
  ],
  "size": [
    8.7685,
    14.6142
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
      0.6931,
      0.7208
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
      0.1875,
      [
        1.0,
        0.8751,
        0.7519,
        0.0652
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.8751,
        0.7519,
        0.1189
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.8751,
        0.7519,
        0.1882
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
    -9.0,
    9.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Sacredlight/n0-8-0-0-0-2.effect"
  }
});
