import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0.0.4.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 32,
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
      "count": 5
    }
  ],
  "duration": 0.0167,
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
    1.25,
    1.5625
  ],
  "heightOverLife": [
    [
      0.0,
      0.5556
    ],
    [
      1.0,
      0.5556
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
        0.7445,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.7445,
        1.0,
        0.0104,
        0.502
      ]
    ],
    [
      0.3333,
      [
        0.7445,
        1.0,
        0.0104,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.7445,
        1.0,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.8671,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.8671,
        1.0,
        0.0478,
        0.7529
      ]
    ],
    [
      0.3333,
      [
        0.8671,
        1.0,
        0.0478,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.8671,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line6"
  },
  "blend": "additive"
});
