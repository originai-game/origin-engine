import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "煙"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    10.0,
    18.0
  ],
  "speedOverLife": [
    [
      0.0,
      3.6937
    ],
    [
      0.25,
      1.6792
    ],
    [
      0.375,
      0.9618
    ],
    [
      0.5,
      0.4505
    ],
    [
      0.625,
      0.1454
    ],
    [
      0.75,
      0.0463
    ],
    [
      0.875,
      0.1533
    ],
    [
      1.0,
      0.4534
    ]
  ],
  "size": [
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.8459
    ],
    [
      0.1562,
      0.7164
    ],
    [
      0.2344,
      0.6084
    ],
    [
      0.3281,
      0.5023
    ],
    [
      0.4688,
      0.3798
    ],
    [
      0.8281,
      0.1439
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
        0.7445,
        0.4591,
        0.0658,
        0.0
      ]
    ],
    [
      0.3281,
      [
        0.7445,
        0.4691,
        0.0658,
        0.0431
      ]
    ],
    [
      1.0,
      [
        0.7445,
        0.5054,
        0.0658,
        0.1961
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.8671,
        0.7012,
        0.0658,
        0.0
      ]
    ],
    [
      0.3594,
      [
        0.8671,
        0.7559,
        0.0658,
        0.0493
      ]
    ],
    [
      1.0,
      [
        0.8671,
        0.9323,
        0.0658,
        0.1961
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Fog3001"
  },
  "blend": "additive"
});
