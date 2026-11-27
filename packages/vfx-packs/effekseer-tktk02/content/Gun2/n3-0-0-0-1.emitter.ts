import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.0.1 "Distortion"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.865,
        0.0
      ],
      [
        -0.6116,
        0.6116,
        0.0
      ],
      [
        -0.865,
        0.0,
        0.0
      ],
      [
        -0.6116,
        -0.6116,
        0.0
      ],
      [
        0.0,
        -0.865,
        0.0
      ],
      [
        0.6116,
        -0.6116,
        0.0
      ],
      [
        0.865,
        0.0,
        0.0
      ],
      [
        0.6116,
        0.6116,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 6
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.0,
    0.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1
    ],
    [
      0.1094,
      0.2948
    ],
    [
      0.2188,
      0.459
    ],
    [
      0.3281,
      0.5955
    ],
    [
      0.4531,
      0.7214
    ],
    [
      0.5781,
      0.8192
    ],
    [
      0.7031,
      0.8933
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.2,
      [
        1.0,
        1.0,
        1.0,
        0.502
      ]
    ],
    [
      0.4,
      [
        1.0,
        1.0,
        1.0,
        0.502
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion2"
  },
  "blend": "additive",
  "distortion": 0.005
});
