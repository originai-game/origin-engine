import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0 "*衝撃生業用"
export default file.particles({
  "maxParticles": 360,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 1080.0,
  "duration": 0.2667,
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
    1.0,
    1.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.4219,
      0.5188
    ],
    [
      0.5312,
      0.5422
    ],
    [
      0.625,
      0.5802
    ],
    [
      0.7344,
      0.6525
    ],
    [
      0.8281,
      0.7441
    ],
    [
      0.9219,
      0.8682
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      2.0
    ],
    [
      0.4219,
      2.1877
    ],
    [
      0.5312,
      2.4218
    ],
    [
      0.625,
      2.8023
    ],
    [
      0.7344,
      3.5253
    ],
    [
      0.8281,
      4.4409
    ],
    [
      0.9219,
      5.6818
    ],
    [
      1.0,
      7.0
    ]
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
        0.586,
        1.0,
        1.0,
        0.1569
      ]
    ],
    [
      0.5,
      [
        0.586,
        1.0,
        1.0,
        0.1569
      ]
    ],
    [
      0.7969,
      [
        0.586,
        1.0,
        1.0,
        0.0976
      ]
    ],
    [
      1.0,
      [
        0.586,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
