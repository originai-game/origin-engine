import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1.0 "炎"
export default file.particles({
  "maxParticles": 2000,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 9600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.25,
    0.75
  ],
  "speed": [
    4.0,
    9.0
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
    10.0,
    10.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.3438,
      0.3408
    ],
    [
      0.5781,
      0.4895
    ],
    [
      0.6875,
      0.5861
    ],
    [
      0.7969,
      0.7054
    ],
    [
      0.9062,
      0.8511
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
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0104,
        0.0023,
        0.098
      ]
    ],
    [
      0.4219,
      [
        1.0,
        0.0308,
        0.0067,
        0.0355
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.098
      ]
    ],
    [
      0.4844,
      [
        1.0,
        0.3728,
        0.0811,
        0.0298
      ]
    ],
    [
      0.7031,
      [
        1.0,
        0.4525,
        0.0985,
        0.0145
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
