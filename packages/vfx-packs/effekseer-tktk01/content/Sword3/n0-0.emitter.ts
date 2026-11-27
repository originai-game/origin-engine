import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "斬撃１"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
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
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.8054
    ],
    [
      0.1562,
      0.6549
    ],
    [
      0.2344,
      0.5427
    ],
    [
      0.3281,
      0.4504
    ],
    [
      0.4688,
      0.3774
    ],
    [
      0.8281,
      0.3288
    ],
    [
      1.0,
      0.25
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      2.5
    ],
    [
      0.0781,
      3.7976
    ],
    [
      0.1562,
      4.8008
    ],
    [
      0.2344,
      5.5486
    ],
    [
      0.3281,
      6.164
    ],
    [
      0.4688,
      6.6509
    ],
    [
      0.8281,
      6.9748
    ],
    [
      1.0,
      7.5
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.1801,
        0.0014,
        0.2084,
        1.0
      ]
    ],
    [
      0.6667,
      [
        0.1801,
        0.0014,
        0.2084,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.1801,
        0.0014,
        0.2084,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
