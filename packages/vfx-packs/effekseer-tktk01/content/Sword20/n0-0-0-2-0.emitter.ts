import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.0 "粒子"
export default file.particles({
  "maxParticles": 1120,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6720.0,
  "duration": 0.1333,
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
    0.125,
    0.125
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1094,
      0.7835
    ],
    [
      0.2188,
      0.6011
    ],
    [
      0.3281,
      0.4494
    ],
    [
      0.4531,
      0.3096
    ],
    [
      0.5781,
      0.2009
    ],
    [
      0.7031,
      0.1185
    ],
    [
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      12.0
    ],
    [
      0.1094,
      13.7318
    ],
    [
      0.2188,
      15.1914
    ],
    [
      0.3281,
      16.4048
    ],
    [
      0.4531,
      17.5234
    ],
    [
      0.5781,
      18.3929
    ],
    [
      0.7031,
      19.0519
    ],
    [
      1.0,
      20.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
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
      0.0333,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
