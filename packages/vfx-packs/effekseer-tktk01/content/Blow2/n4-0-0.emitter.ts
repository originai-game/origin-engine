import { file } from '@origingame/engine/author';
// Effekseer node 4.0.0 "衝撃つぶ"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.5,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 500
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.1333,
    0.3667
  ],
  "speed": [
    2.0,
    26.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1267
    ],
    [
      0.25,
      1.2147
    ],
    [
      0.375,
      0.9106
    ],
    [
      0.5,
      0.7139
    ],
    [
      0.625,
      0.6245
    ],
    [
      0.75,
      0.6424
    ],
    [
      1.0,
      0.9911
    ]
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.8682
    ],
    [
      0.1719,
      0.7441
    ],
    [
      0.2656,
      0.6525
    ],
    [
      0.375,
      0.5802
    ],
    [
      0.4688,
      0.5422
    ],
    [
      0.5781,
      0.5188
    ],
    [
      1.0,
      0.5
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      1.5
    ],
    [
      0.0781,
      1.1045
    ],
    [
      0.1719,
      0.7323
    ],
    [
      0.2656,
      0.4576
    ],
    [
      0.375,
      0.2407
    ],
    [
      0.4688,
      0.1265
    ],
    [
      0.5781,
      0.0563
    ],
    [
      1.0,
      0.0
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
        0.0399,
        0.022,
        0.0
      ]
    ],
    [
      0.3281,
      [
        1.0,
        0.0399,
        0.022,
        0.0
      ]
    ],
    [
      0.3438,
      [
        1.0,
        0.0399,
        0.022,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0399,
        0.022,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
