import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.0 "粒子"
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
    0.3333,
    0.5
  ],
  "speed": [
    7.4,
    74.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1308
    ],
    [
      0.25,
      1.3814
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      0.5423
    ],
    [
      1.0,
      0.4657
    ]
  ],
  "size": [
    0.6167,
    1.5417
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
      0.04,
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
