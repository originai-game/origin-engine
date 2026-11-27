import { file } from '@origingame/engine/author';
// Effekseer node 1.1 "外に伸びる筋glow"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 30
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    12.0,
    12.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1324
    ],
    [
      0.2656,
      1.4047
    ],
    [
      0.5,
      0.8969
    ],
    [
      0.75,
      0.5051
    ],
    [
      1.0,
      0.2705
    ]
  ],
  "size": [
    1.5,
    1.5
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
      2.6667
    ],
    [
      0.1094,
      3.8212
    ],
    [
      0.2188,
      4.7943
    ],
    [
      0.3281,
      5.6032
    ],
    [
      0.4531,
      6.3489
    ],
    [
      0.5781,
      6.9286
    ],
    [
      0.7031,
      7.368
    ],
    [
      1.0,
      8.0
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
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/actionlines"
  },
  "blend": "additive"
});
