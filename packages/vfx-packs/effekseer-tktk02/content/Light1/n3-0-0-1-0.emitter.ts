import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.1.0 "particleColor"
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
    2.925,
    6.975
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
    1.5,
    2.25
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.0478,
        0.0478,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.0228,
        0.0228,
        0.0228,
        0.251
      ]
    ],
    [
      0.5714,
      [
        0.0074,
        0.0074,
        0.0074,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0,
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
        0.5356,
        0.5356,
        0.5356,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.4282,
        0.4282,
        0.4282,
        0.251
      ]
    ],
    [
      0.5714,
      [
        0.3338,
        0.3338,
        0.3338,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
