import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1.0.0.0.3.1.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 263,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 300.0,
  "duration": 0.1,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.05,
    2.85
  ],
  "speedOverLife": [
    [
      0.0,
      4.4254
    ],
    [
      0.2188,
      1.9936
    ],
    [
      0.3281,
      1.1232
    ],
    [
      0.4375,
      0.5008
    ],
    [
      0.6094,
      0.024
    ],
    [
      0.8125,
      0.2502
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
    0.4375,
    0.5625
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.2195,
        1.0,
        0.2195,
        0.502
      ]
    ],
    [
      0.3333,
      [
        0.2195,
        1.0,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5356,
        1.0,
        0.5356,
        0.0
      ]
    ],
    [
      0.1667,
      [
        0.5356,
        1.0,
        0.5356,
        0.7529
      ]
    ],
    [
      0.3333,
      [
        0.5356,
        1.0,
        0.5356,
        0.7529
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
