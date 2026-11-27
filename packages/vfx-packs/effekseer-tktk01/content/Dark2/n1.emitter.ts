import { file } from '@origingame/engine/author';
// Effekseer node 1 "魔法陣衝撃用グロー"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    10.0,
    10.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1
    ],
    [
      0.0938,
      0.2675
    ],
    [
      0.2031,
      0.4315
    ],
    [
      0.3125,
      0.5656
    ],
    [
      0.4219,
      0.6744
    ],
    [
      0.6562,
      0.8416
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
        0.3112,
        0.0278,
        1.0,
        1.0
      ]
    ],
    [
      0.7,
      [
        0.1734,
        0.0108,
        1.0,
        0.3
      ]
    ],
    [
      0.7188,
      [
        0.1703,
        0.0104,
        1.0,
        0.2207
      ]
    ],
    [
      0.75,
      [
        0.1653,
        0.0099,
        1.0,
        0.125
      ]
    ],
    [
      0.7969,
      [
        0.1578,
        0.0091,
        1.0,
        0.0449
      ]
    ],
    [
      0.8594,
      [
        0.1481,
        0.0081,
        1.0,
        0.0075
      ]
    ],
    [
      1.0,
      [
        0.1275,
        0.006,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
