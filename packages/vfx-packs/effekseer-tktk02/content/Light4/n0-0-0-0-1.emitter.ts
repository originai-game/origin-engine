import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1 "ReimsGlow"
export default file.particles({
  "maxParticles": 9,
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
      "count": 7
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
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
      0.2,
      [
        0.4602,
        1.0,
        0.0478,
        0.1255
      ]
    ],
    [
      0.8,
      [
        0.2707,
        1.0,
        0.0478,
        0.1255
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
      0.2,
      [
        0.6997,
        1.0,
        0.0478,
        0.1255
      ]
    ],
    [
      0.8,
      [
        0.5744,
        1.0,
        0.0478,
        0.1255
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
    "$ogRef": "base/vfx/effekseer-tktk02/Light4/n0-0-0-0-1.effect"
  }
});
