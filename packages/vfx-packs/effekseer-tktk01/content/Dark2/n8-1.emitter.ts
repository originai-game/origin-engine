import { file } from '@origingame/engine/author';
// Effekseer node 8.1 "ライン着弾衝撃"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        1.25
      ],
      [
        -0.8839,
        0.0,
        0.8839
      ],
      [
        -1.25,
        0.0,
        0.0
      ],
      [
        -0.8839,
        0.0,
        -0.8839
      ],
      [
        0.0,
        0.0,
        -1.25
      ],
      [
        0.8839,
        0.0,
        -0.8839
      ],
      [
        1.25,
        0.0,
        0.0
      ],
      [
        0.8839,
        0.0,
        0.8839
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 50
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    5.25,
    8.25
  ],
  "speedOverLife": [
    [
      0.0,
      3.1895
    ],
    [
      0.1562,
      2.2241
    ],
    [
      0.2812,
      1.5767
    ],
    [
      0.4688,
      0.8502
    ],
    [
      0.6562,
      0.4174
    ],
    [
      0.6719,
      0.0673
    ],
    [
      0.6875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.0,
    1.0
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
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      -1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.1275,
        0.0037,
        0.3112,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.1275,
        0.0037,
        0.3112,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/wind02"
  },
  "blend": "additive"
});
