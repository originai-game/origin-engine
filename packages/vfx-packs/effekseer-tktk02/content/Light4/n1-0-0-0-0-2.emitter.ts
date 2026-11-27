import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.0.2 "ReimsGlow"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        5.0,
        0.0
      ],
      [
        -3.5355,
        3.5355,
        0.0
      ],
      [
        -5.0,
        0.0,
        0.0
      ],
      [
        -3.5355,
        -3.5355,
        0.0
      ],
      [
        0.0,
        -5.0,
        0.0
      ],
      [
        3.5355,
        -3.5355,
        0.0
      ],
      [
        5.0,
        0.0,
        0.0
      ],
      [
        3.5355,
        3.5355,
        0.0
      ]
    ]
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
    4.1667,
    4.1667
  ],
  "heightOverLife": [
    [
      0.0,
      0.24
    ],
    [
      1.0,
      0.24
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
        0.5356,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.4134,
        1.0,
        0.0478,
        0.251
      ]
    ],
    [
      0.6667,
      [
        0.3081,
        1.0,
        0.0478,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7445,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.6707,
        1.0,
        0.0478,
        0.251
      ]
    ],
    [
      0.6667,
      [
        0.6011,
        1.0,
        0.0478,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-0-0-0-0-2.effect"
  }
});
