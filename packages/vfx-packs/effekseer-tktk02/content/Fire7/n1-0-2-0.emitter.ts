import { file } from '@origingame/engine/author';
// Effekseer node 1.0.2.0 "Fog"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    2.0,
    8.0
  ],
  "speedOverLife": [
    [
      0.0,
      1.4238
    ],
    [
      0.5,
      0.9679
    ],
    [
      1.0,
      0.7016
    ]
  ],
  "size": [
    3.75,
    3.75
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1333
    ],
    [
      0.3125,
      0.5016
    ],
    [
      0.5625,
      0.7486
    ],
    [
      0.7969,
      0.918
    ],
    [
      1.0,
      1.0
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
        0.2195,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.147,
        0.0043,
        0.0941
      ]
    ],
    [
      0.6719,
      [
        1.0,
        0.0892,
        0.0009,
        0.0306
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.4134,
        0.0196,
        0.0941
      ]
    ],
    [
      0.6719,
      [
        1.0,
        0.3066,
        0.0041,
        0.0306
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/FireFog"
  },
  "blend": "additive"
});
