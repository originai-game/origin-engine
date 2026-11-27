import { file } from '@origingame/engine/author';
// Effekseer node 1.0.4 "Glow1"
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
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    4.0,
    4.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.0781,
      0.2636
    ],
    [
      0.1719,
      0.5118
    ],
    [
      0.2656,
      0.6949
    ],
    [
      0.375,
      0.8395
    ],
    [
      0.4688,
      0.9156
    ],
    [
      0.5781,
      0.9625
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
        0.0478,
        0.2195,
        1.0,
        0.0
      ]
    ],
    [
      0.125,
      [
        0.0415,
        0.1905,
        1.0,
        0.4689
      ]
    ],
    [
      0.25,
      [
        0.0356,
        0.1636,
        1.0,
        0.8755
      ]
    ],
    [
      0.4375,
      [
        0.0278,
        0.1275,
        1.0,
        0.5866
      ]
    ],
    [
      0.625,
      [
        0.021,
        0.0963,
        1.0,
        0.3444
      ]
    ],
    [
      0.8125,
      [
        0.0152,
        0.0697,
        1.0,
        0.1488
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.0478,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
