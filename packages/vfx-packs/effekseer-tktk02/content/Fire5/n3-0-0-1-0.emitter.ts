import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0.1.0 "particleGlow"
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
    4.25,
    8.75
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
    1.125,
    1.375
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.1636,
        0.0356,
        0.1255
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.139,
        0.0303,
        0.1569
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0254,
        0.1673
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
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
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.4423,
        0.1636,
        0.1255
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.3993,
        0.139,
        0.1569
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.3587,
        0.1166,
        0.1673
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
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
