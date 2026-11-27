import { file } from '@origingame/engine/author';
// Effekseer node 5.1 "ライン着弾衝撃"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        0.75
      ],
      [
        -0.5303,
        0.0,
        0.5303
      ],
      [
        -0.75,
        0.0,
        0.0
      ],
      [
        -0.5303,
        0.0,
        -0.5303
      ],
      [
        0.0,
        0.0,
        -0.75
      ],
      [
        0.5303,
        0.0,
        -0.5303
      ],
      [
        0.75,
        0.0,
        0.0
      ],
      [
        0.5303,
        0.0,
        0.5303
      ]
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.2083,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    4.5,
    7.5
  ],
  "speedOverLife": [
    [
      0.0,
      0.6982
    ],
    [
      0.1719,
      0.8226
    ],
    [
      0.3281,
      1.2516
    ],
    [
      0.5,
      2.0722
    ],
    [
      0.6562,
      3.1353
    ],
    [
      0.6719,
      0.5344
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
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.125
    ],
    [
      0.0781,
      0.3557
    ],
    [
      0.1719,
      0.5728
    ],
    [
      0.2656,
      0.7331
    ],
    [
      0.375,
      0.8596
    ],
    [
      0.4688,
      0.9262
    ],
    [
      0.5781,
      0.9672
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
