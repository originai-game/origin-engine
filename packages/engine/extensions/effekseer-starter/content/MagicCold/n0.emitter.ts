import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "upright",
    "axis": [
      0.35109999775886536,
      -0.9362999796867371,
      0
    ]
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 70,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0.21950000524520874,
        1,
        1,
        1
      ]
    ],
    [
      0.25,
      [
        0.11659999936819077,
        0.746399998664856,
        1,
        0.75
      ]
    ],
    [
      0.5,
      [
        0.04780000075697899,
        0.5325999855995178,
        1,
        0.5
      ]
    ],
    [
      0.75,
      [
        0.010400000028312206,
        0.35740000009536743,
        1,
        0.25
      ]
    ],
    [
      1,
      [
        0,
        0.21950000524520874,
        1,
        0
      ]
    ]
  ],
  "duration": 1.166700005531311,
  "lifetime": [
    1,
    1
  ],
  "looping": false,
  "maxParticles": 77,
  "randomColorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.1704000234603882,
      2.574899911880493,
      2
    ]
  },
  "size": [
    1,
    1
  ],
  "speed": [
    7.280099868774414,
    12.845199584960938
  ],
  "speedOverLife": [
    [
      0,
      0.2793000042438507
    ],
    [
      0.25,
      0.8626999855041504
    ],
    [
      0.5,
      1.1829999685287476
    ],
    [
      0.75,
      1.228700041770935
    ],
    [
      1,
      1.0056999921798706
    ]
  ],
  "spread": 43.6593017578125,
  "texture": ref("base/vfx/effekseer-starter/textures/SnowCrystals_M")
});
