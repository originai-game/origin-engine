import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "粒子"
export default file.particles({
  "maxParticles": 52,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.299,
      0.001
    ]
  },
  "spread": 29.4962,
  "rate": 120.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    3.8971,
    11.6913
  ],
  "speedOverLife": [
    [
      0.0,
      0.4657
    ],
    [
      0.25,
      0.5423
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      1.3814
    ],
    [
      1.0,
      2.1308
    ]
  ],
  "size": [
    0.5,
    0.5
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
      0.0156,
      [
        1.0,
        1.0,
        1.0,
        0.0209
      ]
    ],
    [
      0.0312,
      [
        1.0,
        1.0,
        1.0,
        0.1605
      ]
    ],
    [
      0.0469,
      [
        1.0,
        1.0,
        1.0,
        0.7847
      ]
    ],
    [
      0.05,
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
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
