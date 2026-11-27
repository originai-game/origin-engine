import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.4 "Glow1"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        1.0,
        0.0
      ],
      [
        -0.7071,
        0.7071,
        0.0
      ],
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
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 7,
      "interval": 0.0667
    }
  ],
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
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
      0.0781,
      0.2636
    ],
    [
      0.1719,
      0.5118
    ],
    [
      0.2656,
      0.6949
    ],
    [
      0.375,
      0.8395
    ],
    [
      0.4688,
      0.9156
    ],
    [
      0.5781,
      0.9625
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
        0.0478,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.0356,
        0.1636,
        1.0,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.0478,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
