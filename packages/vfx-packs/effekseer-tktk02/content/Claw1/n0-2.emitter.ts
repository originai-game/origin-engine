import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "Glow1"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0781,
      [
        1.0,
        0.7605,
        0.7238,
        0.1765
      ]
    ],
    [
      0.1719,
      [
        1.0,
        0.5195,
        0.4554,
        0.3804
      ]
    ],
    [
      0.3281,
      [
        1.0,
        0.2258,
        0.1531,
        0.498
      ]
    ],
    [
      0.3906,
      [
        1.0,
        0.1443,
        0.0797,
        0.4373
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0478,
        0.0104,
        0.3137
      ]
    ],
    [
      0.6719,
      [
        1.0,
        0.0189,
        0.0041,
        0.1235
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
