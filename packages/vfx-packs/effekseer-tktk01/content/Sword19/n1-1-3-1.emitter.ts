import { file } from '@origingame/engine/author';
// Effekseer node 1.1.3.1 "ライン衝撃"
export default file.particles({
  "maxParticles": 313,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 250
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.5,
    5.5
  ],
  "speedOverLife": [
    [
      0.0,
      0.7016
    ],
    [
      0.5,
      0.9679
    ],
    [
      1.0,
      1.4238
    ]
  ],
  "size": [
    0.5,
    0.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      5.0
    ],
    [
      1.0,
      5.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      -1.0,
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
        0.3922
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.3922
      ]
    ]
  ],
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
