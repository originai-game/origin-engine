import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.0 "色付け"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    1.2,
    1.8
  ],
  "speedOverLife": [
    [
      0.0,
      8.385
    ],
    [
      0.0156,
      7.8205
    ],
    [
      0.0625,
      5.7751
    ],
    [
      0.1094,
      4.0776
    ],
    [
      0.1719,
      2.3557
    ],
    [
      0.2344,
      1.2525
    ],
    [
      0.2656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
