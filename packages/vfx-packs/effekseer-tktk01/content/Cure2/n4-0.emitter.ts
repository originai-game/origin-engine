import { file } from '@origingame/engine/author';
// Effekseer node 4.0 "最終星"
export default file.particles({
  "maxParticles": 92,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        6.0
      ],
      [
        -4.2426,
        0.0,
        4.2426
      ],
      [
        -6.0,
        0.0,
        0.0
      ],
      [
        -4.2426,
        0.0,
        -4.2426
      ],
      [
        0.0,
        0.0,
        -6.0
      ],
      [
        4.2426,
        0.0,
        -4.2426
      ],
      [
        6.0,
        0.0,
        0.0
      ],
      [
        4.2426,
        0.0,
        4.2426
      ]
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.15,
    0.45
  ],
  "speed": [
    9.0,
    27.0
  ],
  "size": [
    2.0,
    3.0
  ],
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
