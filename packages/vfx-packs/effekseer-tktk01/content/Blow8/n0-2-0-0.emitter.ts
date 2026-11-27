import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0 "粒"
export default file.particles({
  "maxParticles": 338,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 30
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.1167,
    0.3833
  ],
  "speed": [
    3.5,
    11.5
  ],
  "speedOverLife": [
    [
      0.0,
      1.9349
    ],
    [
      0.2188,
      1.624
    ],
    [
      0.4531,
      1.1824
    ],
    [
      0.9062,
      0.0249
    ],
    [
      1.0,
      0.2521
    ]
  ],
  "size": [
    10.0,
    10.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.0625,
      0.2084
    ],
    [
      0.125,
      0.3708
    ],
    [
      0.2656,
      0.5977
    ],
    [
      0.4062,
      0.6854
    ],
    [
      0.7969,
      0.7391
    ],
    [
      0.9062,
      0.8399
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
        0.0,
        0.0,
        0.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.0,
        0.0,
        0.0,
        0.0588
      ]
    ],
    [
      0.6667,
      [
        0.0,
        0.0,
        0.0,
        0.0588
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.0
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
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0588
      ]
    ],
    [
      0.6667,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0588
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
