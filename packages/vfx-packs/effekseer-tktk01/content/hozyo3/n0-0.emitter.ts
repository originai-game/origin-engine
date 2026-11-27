import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "ナイフ"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        2.0,
        0.0
      ],
      [
        -1.4142,
        1.4142,
        0.0
      ],
      [
        -2.0,
        0.0,
        0.0
      ],
      [
        -1.4142,
        -1.4142,
        0.0
      ],
      [
        0.0,
        -2.0,
        0.0
      ],
      [
        1.4142,
        -1.4142,
        0.0
      ],
      [
        2.0,
        0.0,
        0.0
      ],
      [
        1.4142,
        1.4142,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 6
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.5,
    1.5
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
        1.0
      ]
    ],
    [
      0.25,
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
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/naihu"
  },
  "blend": "additive"
});
