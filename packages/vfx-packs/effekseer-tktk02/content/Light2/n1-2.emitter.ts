import { file } from '@origingame/engine/author';
// Effekseer node 1.2 "Distortion"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
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
      "interval": 0.0167
    }
  ],
  "duration": 0.1167,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    20.0,
    20.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.125
    ],
    [
      0.1094,
      0.3144
    ],
    [
      0.2188,
      0.4741
    ],
    [
      0.3281,
      0.6068
    ],
    [
      0.4531,
      0.7291
    ],
    [
      0.5781,
      0.8242
    ],
    [
      0.7031,
      0.8963
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
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion6"
  },
  "blend": "additive",
  "distortion": 0.004
});
