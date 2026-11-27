import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0 "glow"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        -0.75
      ],
      [
        -0.5303,
        0.0,
        -0.5303
      ],
      [
        -0.75,
        0.0,
        0.0
      ],
      [
        -0.5303,
        0.0,
        0.5303
      ],
      [
        0.0,
        0.0,
        0.75
      ],
      [
        0.5303,
        0.0,
        0.5303
      ],
      [
        0.75,
        0.0,
        0.0
      ],
      [
        0.5303,
        0.0,
        -0.5303
      ]
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.0,
    4.0
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
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
