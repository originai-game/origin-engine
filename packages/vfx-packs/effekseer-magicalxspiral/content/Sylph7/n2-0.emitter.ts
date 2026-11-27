import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "Node"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        -1.1
      ],
      [
        -0.7778,
        0.0,
        -0.7778
      ],
      [
        -1.1,
        0.0,
        0.0
      ],
      [
        -0.7778,
        0.0,
        0.7778
      ],
      [
        0.0,
        0.0,
        1.1
      ],
      [
        0.7778,
        0.0,
        0.7778
      ],
      [
        1.1,
        0.0,
        0.0
      ],
      [
        0.7778,
        0.0,
        -0.7778
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2,
      "cycles": 20,
      "interval": 0.0667
    }
  ],
  "duration": 1.2833,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    3.0,
    3.0
  ],
  "size": [
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.1333
    ],
    [
      1.0,
      0.1333
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.1166,
        0.5356,
        0.1166,
        0.502
      ]
    ],
    [
      0.5,
      [
        0.1166,
        0.5356,
        0.1166,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.5356,
        0.1166,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3587,
        1.0,
        0.3587,
        0.502
      ]
    ],
    [
      0.5,
      [
        0.3587,
        1.0,
        0.3587,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.3587,
        1.0,
        0.3587,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line2"
  },
  "blend": "additive"
});
