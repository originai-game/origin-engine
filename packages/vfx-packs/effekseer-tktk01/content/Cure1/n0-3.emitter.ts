import { file } from '@origingame/engine/author';
// Effekseer node 0.3 "最後の光拡散"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "sphere",
    "radius": 1.0
  },
  "spread": 103.2627,
  "rate": 12000.0,
  "duration": 0.025,
  "looping": false,
  "lifetime": [
    0.0833,
    0.75
  ],
  "speed": [
    10.4614,
    15.7379
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
    1.0,
    1.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.8592,
        0.9323,
        0.0846,
        0.3922
      ]
    ],
    [
      0.2656,
      [
        0.8954,
        0.95,
        0.2221,
        0.288
      ]
    ],
    [
      0.5156,
      [
        0.9303,
        0.9669,
        0.4188,
        0.19
      ]
    ],
    [
      0.7656,
      [
        0.9659,
        0.9839,
        0.6848,
        0.0919
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
  "angularVelocity": [
    -864.0,
    864.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
