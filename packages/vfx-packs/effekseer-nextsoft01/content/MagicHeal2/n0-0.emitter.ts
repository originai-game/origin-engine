import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "柱"
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 9,
      "interval": 0.1667
    }
  ],
  "duration": 1.35,
  "looping": false,
  "lifetime": [
    2.5,
    2.5
  ],
  "speed": [
    2.8,
    2.8
  ],
  "size": [
    1.0,
    1.0
  ],
  "heightOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      16.0
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
      0.0667,
      [
        0.9404,
        1.0,
        0.9404,
        1.0
      ]
    ],
    [
      0.1875,
      [
        0.884,
        1.0,
        0.884,
        1.0
      ]
    ],
    [
      0.4688,
      [
        0.8431,
        1.0,
        0.8431,
        1.0
      ]
    ],
    [
      0.5938,
      [
        0.7915,
        1.0,
        0.7915,
        1.0
      ]
    ],
    [
      0.7344,
      [
        0.6622,
        1.0,
        0.6622,
        1.0
      ]
    ],
    [
      0.8667,
      [
        0.4629,
        1.0,
        0.4629,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Line01"
  },
  "blend": "additive"
});
