import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1 "ライン"
export default file.particles({
  "maxParticles": 1000,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 100
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.5
  ],
  "speed": [
    2.4,
    9.6
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
    0.1,
    0.2
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
      6.6667
    ],
    [
      0.0938,
      6.8528
    ],
    [
      0.2031,
      7.035
    ],
    [
      0.3125,
      7.184
    ],
    [
      0.4219,
      7.3048
    ],
    [
      0.5312,
      7.4021
    ],
    [
      0.6562,
      7.4907
    ],
    [
      1.0,
      7.6667
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
  "rotation": [
    90.0,
    90.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Line01"
  },
  "blend": "additive"
});
