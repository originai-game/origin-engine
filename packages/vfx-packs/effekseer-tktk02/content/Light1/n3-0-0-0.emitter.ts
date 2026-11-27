import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.0 "particleMain"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.125,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    1.4625,
    3.4875
  ],
  "speedOverLife": [
    [
      0.0,
      5.1527
    ],
    [
      0.2031,
      2.1366
    ],
    [
      0.2969,
      1.1693
    ],
    [
      0.4062,
      0.4066
    ],
    [
      0.5469,
      0.0048
    ],
    [
      0.7031,
      0.322
    ],
    [
      0.7344,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.375,
    0.375
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
      0.2857,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.5714,
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
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
