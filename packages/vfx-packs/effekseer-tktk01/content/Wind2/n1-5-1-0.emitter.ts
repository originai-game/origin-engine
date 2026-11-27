import { file } from '@origingame/engine/author';
// Effekseer node 1.5.1.0 "衝撃ライン←"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 15
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    4.5,
    16.5
  ],
  "size": [
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0938,
      0.8139
    ],
    [
      0.2031,
      0.6317
    ],
    [
      0.3125,
      0.4826
    ],
    [
      0.4219,
      0.3618
    ],
    [
      0.5312,
      0.2645
    ],
    [
      0.6562,
      0.176
    ],
    [
      1.0,
      0.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0938,
      1.1241
    ],
    [
      0.2031,
      1.2455
    ],
    [
      0.3125,
      1.3449
    ],
    [
      0.4219,
      1.4254
    ],
    [
      0.5312,
      1.4903
    ],
    [
      0.6562,
      1.5493
    ],
    [
      1.0,
      1.6667
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
        1.0,
        1.0,
        0.8
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.8
      ]
    ]
  ],
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/rainnmidori"
  },
  "blend": "additive"
});
