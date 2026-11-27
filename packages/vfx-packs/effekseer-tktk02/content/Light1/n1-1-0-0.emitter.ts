import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0 "LineMain"
export default file.particles({
  "maxParticles": 7,
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
    0.3333,
    0.3333
  ],
  "speed": [
    16.5,
    16.5
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
    0.9167,
    0.9167
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.75,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
