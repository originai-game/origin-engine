import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.2 "Glow3"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
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
    30.0,
    30.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.05
    ],
    [
      0.1094,
      0.2556
    ],
    [
      0.2188,
      0.429
    ],
    [
      0.3281,
      0.5731
    ],
    [
      0.4531,
      0.7059
    ],
    [
      0.5781,
      0.8092
    ],
    [
      0.7031,
      0.8874
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
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
        0.1098
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.1636,
        0.0356,
        0.1882
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.139,
        0.0303,
        0.2353
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0254,
        0.251
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.0781,
        0.017,
        0.0627
      ]
    ],
    [
      0.875,
      [
        1.0,
        0.0619,
        0.0135,
        0.0157
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
