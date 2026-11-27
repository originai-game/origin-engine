import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.1.0.0.0.0 "Ice"
export default file.particles({
  "maxParticles": 150,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.125,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 600.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.2344,
    0.9375
  ],
  "speedOverLife": [
    [
      0.0,
      9.9519
    ],
    [
      0.2344,
      3.7085
    ],
    [
      0.4062,
      0.0771
    ],
    [
      0.6094,
      3.5919
    ],
    [
      0.8125,
      6.0678
    ],
    [
      0.8281,
      6.4821
    ],
    [
      0.8438,
      7.9176
    ],
    [
      1.0,
      8.5537
    ]
  ],
  "size": [
    0.375,
    0.375
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
        0.3587,
        0.7593,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.2845,
        0.6444,
        1.0,
        1.0
      ]
    ],
    [
      0.4219,
      [
        0.1887,
        0.4882,
        1.0,
        1.0
      ]
    ],
    [
      0.6667,
      [
        0.1166,
        0.3603,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0478,
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
        0.632,
        0.8751,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.547,
        0.8122,
        1.0,
        1.0
      ]
    ],
    [
      0.6667,
      [
        0.3313,
        0.6388,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/6Ice"
  },
  "blend": "additive"
});
