import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0.0.0 "ReimsMain"
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
    0.8333,
    0.8333
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-tktk02/Light4/n1-1-0-0-0-0.effect"
  }
});
