import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.0 "particleMain"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    2.125,
    4.375
  ],
  "speedOverLife": [
    [
      0.0,
      4.9106
    ],
    [
      0.2188,
      1.9764
    ],
    [
      0.3281,
      0.9869
    ],
    [
      0.4219,
      0.4095
    ],
    [
      0.5625,
      0.0123
    ],
    [
      0.7344,
      0.2906
    ],
    [
      0.7656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.25,
    0.25
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
      0.25,
      [
        1.0,
        1.0,
        1.0,
        0.5
      ]
    ],
    [
      0.3125,
      [
        1.0,
        1.0,
        1.0,
        0.5729
      ]
    ],
    [
      0.375,
      [
        1.0,
        1.0,
        1.0,
        0.625
      ]
    ],
    [
      0.4375,
      [
        1.0,
        1.0,
        1.0,
        0.6562
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.6667
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
