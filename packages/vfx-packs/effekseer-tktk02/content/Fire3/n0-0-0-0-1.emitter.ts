import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    1.3125,
    7.6875
  ],
  "speedOverLife": [
    [
      0.0,
      3.6952
    ],
    [
      0.25,
      1.7411
    ],
    [
      0.375,
      1.0269
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      0.1608
    ],
    [
      0.75,
      0.0091
    ],
    [
      0.875,
      0.0448
    ],
    [
      1.0,
      0.2582
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
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0254,
        1.0
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
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.3587,
        0.1166,
        1.0
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/FireFog"
  },
  "blend": "additive"
});
