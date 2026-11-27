import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.5 "Glow2"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.1094,
      0.2165
    ],
    [
      0.2188,
      0.3989
    ],
    [
      0.3281,
      0.5506
    ],
    [
      0.4531,
      0.6904
    ],
    [
      0.5781,
      0.7991
    ],
    [
      0.7031,
      0.8815
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.0233,
        0.0104,
        0.1255
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0175,
        0.0104,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
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
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.1562,
      [
        1.0,
        0.1538,
        0.0335,
        0.0588
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.107,
        0.0233,
        0.1255
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0802,
        0.0175,
        0.1255
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
