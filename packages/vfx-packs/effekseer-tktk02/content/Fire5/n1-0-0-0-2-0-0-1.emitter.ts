import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.2.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 188,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    0.7339,
    1.3982
  ],
  "speedOverLife": [
    [
      0.0,
      5.8021
    ],
    [
      0.1875,
      2.2912
    ],
    [
      0.2812,
      1.1208
    ],
    [
      0.375,
      0.3714
    ],
    [
      0.4844,
      0.029
    ],
    [
      0.6562,
      0.2173
    ],
    [
      0.6875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.4375,
    0.5625
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
      0.1667,
      [
        1.0,
        0.1813,
        0.0395,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.09,
        0.0196,
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
      0.1667,
      [
        1.0,
        0.4723,
        0.1813,
        1.0
      ]
    ],
    [
      0.6667,
      [
        1.0,
        0.3081,
        0.09,
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
