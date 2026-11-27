import { file } from '@origingame/engine/author';
// Effekseer node 1.3 "Distortion"
export default file.particles({
  "maxParticles": 13,
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
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    8.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.25
    ],
    [
      0.0781,
      0.4477
    ],
    [
      0.1719,
      0.6339
    ],
    [
      0.2656,
      0.7712
    ],
    [
      0.375,
      0.8797
    ],
    [
      0.4688,
      0.9367
    ],
    [
      0.5781,
      0.9718
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
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.0418
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.0753
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion"
  },
  "blend": "additive",
  "distortion": 0.02
});
