import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "衝撃粒"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.0576,
      0.9983,
      0.001
    ]
  },
  "spread": 6.1605,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 200
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.1667,
    0.6667
  ],
  "speed": [
    7.225,
    24.03
  ],
  "speedOverLife": [
    [
      0.0,
      1.801
    ],
    [
      0.25,
      1.1733
    ],
    [
      0.5,
      0.8138
    ],
    [
      0.75,
      0.7338
    ],
    [
      1.0,
      0.9279
    ]
  ],
  "size": [
    1.0,
    1.0
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
        1.0,
        0.1275,
        0.1275,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1275,
        0.1275,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.586,
        0.586,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.586,
        0.586,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
