import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.2 "Glow"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        1.5,
        0.0
      ],
      [
        -1.0607,
        1.0607,
        0.0
      ],
      [
        -1.5,
        0.0,
        0.0
      ],
      [
        -1.0607,
        -1.0607,
        0.0
      ],
      [
        0.0,
        -1.5,
        0.0
      ],
      [
        1.0607,
        -1.0607,
        0.0
      ],
      [
        1.5,
        0.0,
        0.0
      ],
      [
        1.0607,
        1.0607,
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
    0.8333,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    10.7143,
    14.2857
  ],
  "sizeOverLife": [
    [
      0.0,
      0.14
    ],
    [
      0.1094,
      0.3262
    ],
    [
      0.2188,
      0.4831
    ],
    [
      0.3281,
      0.6135
    ],
    [
      0.4531,
      0.7338
    ],
    [
      0.5781,
      0.8272
    ],
    [
      0.7031,
      0.8981
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.3,
      [
        1.0,
        0.1849,
        0.0402,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.3,
      [
        1.0,
        0.4248,
        0.1535,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
