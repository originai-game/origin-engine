import { file } from '@origingame/engine/author';
// Effekseer node 1 "yuki2"
export default file.particles({
  "maxParticles": 64,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.341,
      2.4903,
      2.0
    ]
  },
  "spread": 34.9679,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 80,
      "interval": 0.0167
    }
  ],
  "duration": 1.3333,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    10.8,
    17.3482
  ],
  "speedOverLife": [
    [
      0.0,
      0.2793
    ],
    [
      0.25,
      0.8627
    ],
    [
      0.5,
      1.183
    ],
    [
      0.75,
      1.2287
    ],
    [
      1.0,
      1.0057
    ]
  ],
  "size": [
    0.3,
    0.3
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.2873,
      -0.9578,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.25,
      [
        0.1166,
        0.7464,
        1.0,
        0.75
      ]
    ],
    [
      0.5,
      [
        0.0478,
        0.5326,
        1.0,
        0.5
      ]
    ],
    [
      0.75,
      [
        0.0104,
        0.3574,
        1.0,
        0.25
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.2195,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/SnowCrystals_S"
  },
  "blend": "additive"
});
