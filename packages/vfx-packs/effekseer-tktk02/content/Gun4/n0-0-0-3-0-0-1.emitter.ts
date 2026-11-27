import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.3.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 2438,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 1200.0,
  "duration": 0.0625,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.25,
    2.75
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
    0.875,
    1.125
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
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.502
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
        0.502
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.3204,
        0.0963,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
