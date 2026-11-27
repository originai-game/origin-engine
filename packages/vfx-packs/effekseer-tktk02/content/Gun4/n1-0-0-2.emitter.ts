import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.2 "Glow"
export default file.particles({
  "maxParticles": 10,
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
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.25
    ],
    [
      0.1094,
      0.4124
    ],
    [
      0.2188,
      0.5492
    ],
    [
      0.3281,
      0.663
    ],
    [
      0.4531,
      0.7678
    ],
    [
      0.7031,
      0.9111
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
      0.4,
      [
        1.0,
        0.1741,
        0.0379,
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
      0.4,
      [
        1.0,
        0.391,
        0.1344,
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
