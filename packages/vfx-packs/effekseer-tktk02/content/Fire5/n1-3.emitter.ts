import { file } from '@origingame/engine/author';
// Effekseer node 1.3 "Glow"
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
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.0938,
      [
        1.0,
        0.1975,
        0.043,
        0.2681
      ]
    ],
    [
      0.1719,
      [
        1.0,
        0.1801,
        0.0392,
        0.4715
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.147,
        0.032,
        0.834
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0254,
        0.5632
      ]
    ],
    [
      0.6719,
      [
        1.0,
        0.0892,
        0.0194,
        0.3275
      ]
    ],
    [
      0.8281,
      [
        1.0,
        0.0677,
        0.0147,
        0.1515
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
