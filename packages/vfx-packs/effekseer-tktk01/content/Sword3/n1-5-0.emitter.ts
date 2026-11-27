import { file } from '@origingame/engine/author';
// Effekseer node 1.5.0 "粒収縮"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.1541,
      0.4757,
      0.001
    ]
  },
  "spread": 1.8468,
  "rate": 1200.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.3667,
    0.5333
  ],
  "speed": [
    10.6283,
    12.7414
  ],
  "speedOverLife": [
    [
      0.0,
      2.0637
    ],
    [
      0.25,
      1.1969
    ],
    [
      0.4844,
      0.7543
    ],
    [
      0.7344,
      0.674
    ],
    [
      0.8594,
      0.7764
    ],
    [
      1.0,
      1.0021
    ]
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0278,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      0.4815,
      [
        0.0278,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0278,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3112,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      0.4815,
      [
        0.3112,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.3112,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
