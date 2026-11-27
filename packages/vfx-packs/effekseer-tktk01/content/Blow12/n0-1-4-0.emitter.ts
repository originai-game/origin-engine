import { file } from '@origingame/engine/author';
// Effekseer node 0.1.4.0 "ライン"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    3.0,
    3.0
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
    1.0,
    1.0
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
      4.0
    ],
    [
      0.0781,
      5.0546
    ],
    [
      0.1719,
      6.0472
    ],
    [
      0.2656,
      6.7797
    ],
    [
      0.375,
      7.3582
    ],
    [
      0.4688,
      7.6625
    ],
    [
      0.5781,
      7.8498
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
        1.0,
        1.0,
        1.0,
        0.502
      ]
    ],
    [
      0.1719,
      [
        1.0,
        1.0,
        1.0,
        0.7555
      ]
    ],
    [
      0.6562,
      [
        1.0,
        1.0,
        1.0,
        0.9949
      ]
    ],
    [
      0.6719,
      [
        1.0,
        1.0,
        1.0,
        0.9844
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
