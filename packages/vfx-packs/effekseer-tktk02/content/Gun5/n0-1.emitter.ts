import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "Glow"
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
      0.1094,
      0.3209
    ],
    [
      0.2188,
      0.4791
    ],
    [
      0.3281,
      0.6105
    ],
    [
      0.4531,
      0.7317
    ],
    [
      0.5781,
      0.8259
    ],
    [
      0.7031,
      0.8973
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
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.4647,
        0.1835,
        0.4532
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.3993,
        0.1511,
        0.8127
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.2845,
        0.0963,
        0.6255
      ]
    ],
    [
      0.625,
      [
        1.0,
        0.2349,
        0.0738,
        0.3989
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.1905,
        0.0546,
        0.2191
      ]
    ],
    [
      0.875,
      [
        1.0,
        0.1511,
        0.0385,
        0.0862
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
