import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0 "粒収縮"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.3667,
    0.5333
  ],
  "speed": [
    10.0,
    12.2222
  ],
  "speedOverLife": [
    [
      0.0,
      0.7049
    ],
    [
      0.25,
      0.9438
    ],
    [
      0.5,
      1.0749
    ],
    [
      0.75,
      1.0937
    ],
    [
      1.0,
      1.0023
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
        0.0278,
        0.0,
        0.0278,
        0.5882
      ]
    ],
    [
      0.2593,
      [
        0.0278,
        0.0,
        0.0278,
        0.5882
      ]
    ],
    [
      1.0,
      [
        0.0278,
        0.0,
        0.0278,
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
        0.3112,
        0.5882
      ]
    ],
    [
      0.2593,
      [
        0.3112,
        0.0,
        0.3112,
        0.5882
      ]
    ],
    [
      1.0,
      [
        0.3112,
        0.0,
        0.3112,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
