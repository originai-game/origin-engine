import { file } from '@origingame/engine/author';
// Effekseer node 5.0 "煙メイン"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.4667,
    0.4667
  ],
  "speed": [
    0.0,
    7.5
  ],
  "speedOverLife": [
    [
      0.0,
      2.1308
    ],
    [
      0.25,
      1.3814
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      0.5423
    ],
    [
      1.0,
      0.4657
    ]
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1562,
      0.9031
    ],
    [
      0.3281,
      0.7509
    ],
    [
      1.0,
      0.0
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
        0.8277,
        0.4423,
        0.0802,
        0.1961
      ]
    ],
    [
      0.2656,
      [
        0.8718,
        0.5667,
        0.2166,
        0.144
      ]
    ],
    [
      0.5156,
      [
        0.9143,
        0.6993,
        0.4136,
        0.095
      ]
    ],
    [
      0.7656,
      [
        0.958,
        0.8472,
        0.6816,
        0.046
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
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
