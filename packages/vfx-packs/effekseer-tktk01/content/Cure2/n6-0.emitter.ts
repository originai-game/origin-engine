import { file } from '@origingame/engine/author';
// Effekseer node 6.0 "中心からの光"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      1.0,
      1.0
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.2083,
  "looping": false,
  "lifetime": [
    0.1667,
    0.5
  ],
  "speed": [
    30.0,
    30.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1267
    ],
    [
      0.25,
      1.2147
    ],
    [
      0.375,
      0.9107
    ],
    [
      0.5,
      0.7139
    ],
    [
      0.625,
      0.6245
    ],
    [
      0.75,
      0.6424
    ],
    [
      1.0,
      0.9911
    ]
  ],
  "size": [
    2.0,
    2.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.5882
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
