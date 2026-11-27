import { file } from '@origingame/engine/author';
// Effekseer node 3.0 "glow"
export default file.particles({
  "maxParticles": 75,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        -1.5
      ],
      [
        -1.0607,
        0.0,
        -1.0607
      ],
      [
        -1.5,
        0.0,
        0.0
      ],
      [
        -1.0607,
        0.0,
        1.0607
      ],
      [
        0.0,
        0.0,
        1.5
      ],
      [
        1.0607,
        0.0,
        1.0607
      ],
      [
        1.5,
        0.0,
        0.0
      ],
      [
        1.0607,
        0.0,
        -1.0607
      ]
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    6.0,
    18.0
  ],
  "speedOverLife": [
    [
      0.0,
      12.0339
    ],
    [
      0.0625,
      6.7405
    ],
    [
      0.1094,
      3.6016
    ],
    [
      0.1562,
      1.4393
    ],
    [
      0.2031,
      0.2536
    ],
    [
      0.2969,
      0.5236
    ],
    [
      0.3125,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.75,
    0.75
  ],
  "heightOverLife": [
    [
      0.0,
      4.0
    ],
    [
      1.0,
      4.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.0478,
        1.0,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0478,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.2195,
        1.0,
        0.3922
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
