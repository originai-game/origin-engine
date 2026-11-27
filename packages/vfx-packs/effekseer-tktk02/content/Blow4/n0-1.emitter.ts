import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "Grow"
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
    7.5,
    7.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1333
    ],
    [
      0.0781,
      0.3618
    ],
    [
      0.1719,
      0.5769
    ],
    [
      0.2656,
      0.7356
    ],
    [
      0.375,
      0.8609
    ],
    [
      0.4688,
      0.9269
    ],
    [
      0.5781,
      0.9675
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
      0.125,
      [
        1.0,
        0.1905,
        0.0415,
        0.2821
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.1636,
        0.0356,
        0.6265
      ]
    ],
    [
      0.4375,
      [
        1.0,
        0.1275,
        0.0278,
        0.5399
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.0963,
        0.021,
        0.4066
      ]
    ],
    [
      0.8125,
      [
        1.0,
        0.0697,
        0.0152,
        0.2267
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
