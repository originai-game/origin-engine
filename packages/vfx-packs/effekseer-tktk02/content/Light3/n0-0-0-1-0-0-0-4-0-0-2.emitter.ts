import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1.0.0.0.4.0.0.2 "LineGlow"
export default file.particles({
  "maxParticles": 175,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 300.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.75,
    5.25
  ],
  "speedOverLife": [
    [
      0.0,
      2.5559
    ],
    [
      0.2344,
      1.64
    ],
    [
      0.4531,
      0.9806
    ],
    [
      0.625,
      0.6061
    ],
    [
      0.8125,
      0.3417
    ],
    [
      0.8281,
      0.273
    ],
    [
      0.8438,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    2.5,
    3.75
  ],
  "heightOverLife": [
    [
      0.0,
      0.3
    ],
    [
      1.0,
      0.3
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.2195,
        1.0,
        0.2195,
        0.1673
      ]
    ],
    [
      0.5,
      [
        0.2195,
        1.0,
        0.2195,
        0.1882
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5356,
        1.0,
        0.5356,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.5356,
        1.0,
        0.5356,
        0.1673
      ]
    ],
    [
      0.5,
      [
        0.5356,
        1.0,
        0.5356,
        0.1882
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line6"
  },
  "blend": "additive"
});
