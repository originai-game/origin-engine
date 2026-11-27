import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.4 "Glow"
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.1,
      0.1,
      0.1
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 32,
      "interval": 0.0333
    }
  ],
  "duration": 1.05,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    10.0,
    10.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1
    ],
    [
      0.1094,
      0.2948
    ],
    [
      0.2188,
      0.459
    ],
    [
      0.3281,
      0.5955
    ],
    [
      0.4531,
      0.7214
    ],
    [
      0.5781,
      0.8192
    ],
    [
      0.7031,
      0.8933
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
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.0781,
      [
        1.0,
        0.2011,
        0.0438,
        0.1042
      ]
    ],
    [
      0.1562,
      [
        1.0,
        0.1835,
        0.0399,
        0.183
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.1636,
        0.0356,
        0.2471
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.147,
        0.032,
        0.2789
      ]
    ],
    [
      0.6719,
      [
        1.0,
        0.0892,
        0.0194,
        0.1094
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
