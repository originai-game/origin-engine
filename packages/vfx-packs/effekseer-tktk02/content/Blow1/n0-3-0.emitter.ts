import { file } from '@origingame/engine/author';
// Effekseer node 0.3.0 "Glow"
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
      "count": 10
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
    3.125,
    3.125
  ],
  "sizeOverLife": [
    [
      0.0,
      0.16
    ],
    [
      0.1094,
      0.3418
    ],
    [
      0.2188,
      0.4951
    ],
    [
      0.3281,
      0.6225
    ],
    [
      0.4531,
      0.74
    ],
    [
      0.5781,
      0.8313
    ],
    [
      0.7031,
      0.9005
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.6888,
        0.6337,
        0.0314
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.5566,
        0.4841,
        0.1176
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.4396,
        0.3562,
        0.251
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.2498,
        0.1628,
        0.251
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0478,
        0.251
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.0254,
        0.0104,
        0.0627
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
