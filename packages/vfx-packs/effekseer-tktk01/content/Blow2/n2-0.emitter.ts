import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "ビルボード飛び散り"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 100
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3667,
    0.4667
  ],
  "speed": [
    3.6,
    20.4
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
  "colorOverLife": [
    [
      0.0,
      [
        0.586,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      0.8,
      [
        0.586,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.586,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7742,
        0.1275,
        0.1275,
        1.0
      ]
    ],
    [
      0.8,
      [
        0.7742,
        0.1275,
        0.1275,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.7742,
        0.1275,
        0.1275,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
