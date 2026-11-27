import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.0 "LineMain"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    3.5357,
    10.6071
  ],
  "speedOverLife": [
    [
      0.0,
      5.2564
    ],
    [
      0.2031,
      2.1303
    ],
    [
      0.2969,
      1.1396
    ],
    [
      0.4062,
      0.3723
    ],
    [
      0.5312,
      0.0079
    ],
    [
      0.6875,
      0.3212
    ],
    [
      0.7188,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.9167,
    0.9167
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
      0.2,
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
