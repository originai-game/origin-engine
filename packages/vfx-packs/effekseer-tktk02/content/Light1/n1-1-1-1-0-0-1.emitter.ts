import { file } from '@origingame/engine/author';
// Effekseer node 1.1.1.1.0.0.1 "LineColor"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.5,
        0.0
      ],
      [
        -0.3536,
        0.3536,
        0.0
      ],
      [
        -0.5,
        0.0,
        0.0
      ],
      [
        -0.3536,
        -0.3536,
        0.0
      ],
      [
        0.0,
        -0.5,
        0.0
      ],
      [
        0.3536,
        -0.3536,
        0.0
      ],
      [
        0.5,
        0.0,
        0.0
      ],
      [
        0.3536,
        0.3536,
        0.0
      ]
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
    1.8333,
    1.8333
  ],
  "speedOverLife": [
    [
      0.0,
      2.1324
    ],
    [
      0.2656,
      1.4047
    ],
    [
      0.5,
      0.8969
    ],
    [
      0.75,
      0.5051
    ],
    [
      1.0,
      0.2705
    ]
  ],
  "size": [
    3.6667,
    3.6667
  ],
  "heightOverLife": [
    [
      0.0,
      0.55
    ],
    [
      1.0,
      0.55
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.9116,
        1.0,
        0.0478,
        1.0
      ]
    ],
    [
      0.6667,
      [
        0.673,
        1.0,
        0.0478,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
