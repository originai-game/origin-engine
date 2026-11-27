import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "粒子"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    1.5,
    4.5
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
    0.2,
    0.3
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
      0.0333,
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
        0.5882
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
