import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "衝撃つぶ"
export default file.particles({
  "maxParticles": 4096,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.2,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1000
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.1333,
    0.3667
  ],
  "speed": [
    6.0,
    15.6
  ],
  "speedOverLife": [
    [
      0.0,
      3.6952
    ],
    [
      0.25,
      1.7411
    ],
    [
      0.375,
      1.0269
    ],
    [
      0.5,
      0.5001
    ],
    [
      0.625,
      0.1608
    ],
    [
      0.75,
      0.0091
    ],
    [
      0.875,
      0.0448
    ],
    [
      1.0,
      0.2582
    ]
  ],
  "size": [
    1.0,
    1.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.0781,
      0.3682
    ],
    [
      0.1719,
      0.2441
    ],
    [
      0.2656,
      0.1525
    ],
    [
      0.375,
      0.0802
    ],
    [
      0.5781,
      0.0188
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
        0.4045,
        0.1701,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.4045,
        0.1701,
        0.0
      ]
    ],
    [
      0.3438,
      [
        1.0,
        0.4045,
        0.1701,
        0.9844
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.4045,
        0.1701,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
