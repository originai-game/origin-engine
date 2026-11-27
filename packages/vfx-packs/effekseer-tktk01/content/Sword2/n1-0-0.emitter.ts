import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "爆発中心"
export default file.particles({
  "maxParticles": 1875,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.0128,
      0.0635,
      0.0048
    ]
  },
  "spread": 41.7015,
  "rate": 8000.0,
  "duration": 0.1875,
  "looping": false,
  "lifetime": [
    0.3333,
    0.5
  ],
  "speed": [
    2.4615,
    11.7318
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
    2.625,
    2.625
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1429
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      -0.2383,
      0.4782,
      0.8453
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.8046,
        0.0329,
        0.0329,
        0.0
      ]
    ],
    [
      0.16,
      [
        0.8111,
        0.0412,
        0.0328,
        0.1809
      ]
    ],
    [
      0.5312,
      [
        0.8426,
        0.0949,
        0.0323,
        0.1087
      ]
    ],
    [
      1.0,
      [
        0.8913,
        0.2233,
        0.0316,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
