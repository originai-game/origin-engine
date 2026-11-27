import { file } from '@origingame/engine/author';
// Effekseer node 5.1.0.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 188,
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
  "duration": 0.0208,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    2.25,
    6.75
  ],
  "speedOverLife": [
    [
      0.0,
      2.9785
    ],
    [
      0.1562,
      2.1316
    ],
    [
      0.2969,
      1.4897
    ],
    [
      0.5,
      0.7996
    ],
    [
      0.7031,
      0.3896
    ],
    [
      0.7188,
      0.0809
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
    2.1875,
    2.8125
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
      0.2857,
      [
        1.0,
        0.1564,
        0.034,
        0.7529
      ]
    ],
    [
      0.4286,
      [
        1.0,
        0.1291,
        0.0281,
        0.7529
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
      0.2857,
      [
        1.0,
        0.4298,
        0.1564,
        0.7529
      ]
    ],
    [
      0.4286,
      [
        1.0,
        0.3816,
        0.1291,
        0.7529
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
