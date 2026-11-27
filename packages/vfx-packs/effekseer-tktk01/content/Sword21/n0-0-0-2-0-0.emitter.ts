import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.0.0 "粒子色付け"
export default file.particles({
  "maxParticles": 325,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      7.7083,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 1200.0,
  "duration": 0.2167,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    2.59,
    25.9
  ],
  "speedOverLife": [
    [
      0.0,
      8.361
    ],
    [
      0.0625,
      5.5279
    ],
    [
      0.1094,
      3.8398
    ],
    [
      0.1406,
      3.0007
    ],
    [
      0.1719,
      2.3906
    ],
    [
      0.2344,
      1.8577
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
    2.1583,
    2.1583
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
