import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.1.1.0 "LineMain"
export default file.particles({
  "maxParticles": 6,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        -1.0,
        0.0,
        0.0
      ],
      [
        -0.7071,
        -0.7071,
        0.0
      ],
      [
        0.0,
        -1.0,
        0.0
      ],
      [
        0.7071,
        -0.7071,
        0.0
      ],
      [
        1.0,
        0.0,
        0.0
      ],
      [
        0.7071,
        0.7071,
        0.0
      ],
      [
        0.0,
        1.0,
        0.0
      ],
      [
        -0.7071,
        0.7071,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0667
    }
  ],
  "duration": 0.2833,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    16.5,
    16.5
  ],
  "speedOverLife": [
    [
      0.0,
      1.7666
    ],
    [
      0.5156,
      0.931
    ],
    [
      1.0,
      0.4209
    ]
  ],
  "size": [
    4.5833,
    4.5833
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
      0.3333,
      [
        1.0,
        1.0,
        0.0478,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        1.0,
        0.0478,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
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
