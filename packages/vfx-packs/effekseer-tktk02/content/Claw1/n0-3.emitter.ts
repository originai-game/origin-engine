import { file } from '@origingame/engine/author';
// Effekseer node 0.3 "Glow_Distortion"
export default file.particles({
  "maxParticles": 7,
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
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
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
    20.0,
    20.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.05
    ],
    [
      0.1094,
      0.2556
    ],
    [
      0.2188,
      0.429
    ],
    [
      0.3281,
      0.5731
    ],
    [
      0.4531,
      0.7059
    ],
    [
      0.5781,
      0.8092
    ],
    [
      0.7031,
      0.8874
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
      0.0781,
      [
        1.0,
        1.0,
        1.0,
        0.2161
      ]
    ],
    [
      0.1719,
      [
        1.0,
        1.0,
        1.0,
        0.427
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        0.5625
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        0.6667
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
  "distortion": 0.1
});
