import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.1 "Wind_Sprite"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.4,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2,
      "cycles": 20,
      "interval": 0.0417
    }
  ],
  "duration": 0.8083,
  "looping": false,
  "lifetime": [
    1.5,
    1.5
  ],
  "speed": [
    0.42,
    1.02
  ],
  "size": [
    7.5,
    8.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.25,
      0.5366
    ],
    [
      0.375,
      0.3762
    ],
    [
      0.5,
      0.2631
    ],
    [
      0.625,
      0.1976
    ],
    [
      0.75,
      0.1795
    ],
    [
      0.875,
      0.2088
    ],
    [
      1.0,
      0.2856
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      -1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.7519,
        0.6388,
        0.4423,
        0.0
      ]
    ],
    [
      0.6667,
      [
        0.7519,
        0.6388,
        0.4423,
        0.75
      ]
    ],
    [
      0.7188,
      [
        0.7519,
        0.6388,
        0.4423,
        0.6823
      ]
    ],
    [
      0.7812,
      [
        0.7519,
        0.6388,
        0.4423,
        0.5768
      ]
    ],
    [
      0.8889,
      [
        0.7519,
        0.6388,
        0.4423,
        0.3333
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.6388,
        0.4423,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Wind"
  },
  "blend": "alpha"
});
