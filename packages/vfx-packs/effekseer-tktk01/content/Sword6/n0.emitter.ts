import { file } from '@origingame/engine/author';
// Effekseer node 0 "メイン斬撃"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 10
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
    1.5,
    1.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.5
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      4.0
    ],
    [
      1.0,
      8.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.3746,
      0.9272,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.9075,
        0.2632,
        0.9406,
        0.1961
      ]
    ],
    [
      0.5,
      [
        0.9075,
        0.2632,
        0.9406,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.9075,
        0.2632,
        0.9406,
        0.0
      ]
    ]
  ],
  "rotation": [
    -22.0,
    -22.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
