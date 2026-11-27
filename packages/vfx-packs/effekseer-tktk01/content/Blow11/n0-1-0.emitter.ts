import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "レンズフレア"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        -5.25,
        0.0
      ],
      [
        3.7123,
        -3.7123,
        0.0
      ],
      [
        5.25,
        0.0,
        0.0
      ],
      [
        3.7123,
        3.7123,
        0.0
      ],
      [
        0.0,
        5.25,
        0.0
      ],
      [
        -3.7123,
        3.7123,
        0.0
      ],
      [
        -5.25,
        0.0,
        0.0
      ],
      [
        -3.7123,
        -3.7123,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 10
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    5.5,
    5.5
  ],
  "speedOverLife": [
    [
      0.0,
      2.1308
    ],
    [
      0.25,
      1.3814
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      0.5423
    ],
    [
      1.0,
      0.4657
    ]
  ],
  "size": [
    0.25,
    0.25
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0938,
      0.8139
    ],
    [
      0.2031,
      0.6317
    ],
    [
      0.3125,
      0.4826
    ],
    [
      0.4219,
      0.3618
    ],
    [
      0.5312,
      0.2645
    ],
    [
      0.6562,
      0.176
    ],
    [
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      40.0
    ],
    [
      0.0938,
      36.2775
    ],
    [
      0.2031,
      32.634
    ],
    [
      0.3125,
      29.6524
    ],
    [
      0.4219,
      27.2367
    ],
    [
      0.5312,
      25.291
    ],
    [
      0.6562,
      23.5198
    ],
    [
      1.0,
      20.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
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
      0.0667,
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
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/actionlines"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk01/Blow11/n0-1-0.effect"
  }
});
