import { file } from '@origingame/engine/author';
// Effekseer node 2 "クロス１"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.7364
    ],
    [
      0.1719,
      0.4882
    ],
    [
      0.2656,
      0.3051
    ],
    [
      0.375,
      0.1605
    ],
    [
      0.4688,
      0.0844
    ],
    [
      0.5781,
      0.0375
    ],
    [
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      2.0
    ],
    [
      0.0781,
      3.2303
    ],
    [
      0.1719,
      4.3885
    ],
    [
      0.2656,
      5.243
    ],
    [
      0.375,
      5.9179
    ],
    [
      0.4688,
      6.273
    ],
    [
      0.5781,
      6.4915
    ],
    [
      1.0,
      6.6667
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
        0.1011,
        1.0,
        1.0,
        0.3922
      ]
    ],
    [
      0.5,
      [
        0.1011,
        1.0,
        1.0,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.1011,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -70.0,
    -70.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
