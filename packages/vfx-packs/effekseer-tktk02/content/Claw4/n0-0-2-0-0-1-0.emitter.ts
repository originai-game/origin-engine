import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2.0.0.1.0 "LineGlow"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.375,
      0.001
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
    0.4167
  ],
  "speed": [
    4.5156,
    13.5469
  ],
  "speedOverLife": [
    [
      0.0,
      3.6952
    ],
    [
      0.25,
      1.7411
    ],
    [
      0.375,
      1.0269
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      0.1608
    ],
    [
      0.75,
      0.0091
    ],
    [
      0.875,
      0.0448
    ],
    [
      1.0,
      0.2582
    ]
  ],
  "size": [
    3.0104,
    4.5156
  ],
  "heightOverLife": [
    [
      0.0,
      0.5
    ],
    [
      1.0,
      0.5
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
        1.0,
        0.5356,
        1.0,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.6997,
        0.391,
        1.0,
        0.251
      ]
    ],
    [
      0.4062,
      [
        0.4501,
        0.2674,
        1.0,
        0.251
      ]
    ],
    [
      0.6,
      [
        0.269,
        0.1741,
        1.0,
        0.251
      ]
    ],
    [
      0.7969,
      [
        0.1355,
        0.1011,
        1.0,
        0.1275
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0478,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7445,
        1.0,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.8945,
        0.6997,
        1.0,
        0.251
      ]
    ],
    [
      0.6,
      [
        0.7026,
        0.6147,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
