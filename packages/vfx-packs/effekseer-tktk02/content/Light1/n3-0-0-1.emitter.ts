import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.1 "particleColor"
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
    0.6562,
    0.8438
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.5356,
        1.0,
        0.0104,
        0.7529
      ]
    ],
    [
      0.5714,
      [
        0.5356,
        1.0,
        0.0104,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.7445,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.7445,
        1.0,
        0.0478,
        0.7529
      ]
    ],
    [
      0.5714,
      [
        0.7445,
        1.0,
        0.0478,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.7445,
        1.0,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
