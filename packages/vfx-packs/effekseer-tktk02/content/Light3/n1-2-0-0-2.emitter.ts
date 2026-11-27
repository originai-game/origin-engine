import { file } from '@origingame/engine/author';
// Effekseer node 1.2.0.0.2 "particleGlow"
export default file.particles({
  "maxParticles": 150,
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
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.25,
    3.15
  ],
  "speedOverLife": [
    [
      0.0,
      2.5559
    ],
    [
      0.2344,
      1.64
    ],
    [
      0.4531,
      0.9806
    ],
    [
      0.625,
      0.6061
    ],
    [
      0.8125,
      0.3417
    ],
    [
      0.8281,
      0.273
    ],
    [
      0.8438,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.0,
    1.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        0.0104,
        0.1673
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        0.0104,
        0.1882
      ]
    ],
    [
      1.0,
      [
        1.0,
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
        1.0,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        0.0478,
        0.1673
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        0.0478,
        0.1882
      ]
    ],
    [
      1.0,
      [
        1.0,
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
