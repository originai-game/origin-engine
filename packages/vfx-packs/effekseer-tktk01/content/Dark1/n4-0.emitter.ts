import { file } from '@origingame/engine/author';
// Effekseer node 4.0 "中心衝撃"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.1667,
    0.5
  ],
  "speed": [
    7.5,
    13.5
  ],
  "speedOverLife": [
    [
      0.0,
      2.1324
    ],
    [
      0.2656,
      1.4047
    ],
    [
      0.5,
      0.8969
    ],
    [
      0.75,
      0.5051
    ],
    [
      1.0,
      0.2705
    ]
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.25
    ],
    [
      0.0781,
      0.4469
    ],
    [
      0.1719,
      0.6302
    ],
    [
      0.2656,
      0.7635
    ],
    [
      0.375,
      0.8666
    ],
    [
      0.5625,
      0.9479
    ],
    [
      0.875,
      0.9726
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
        0.0278,
        0.1275,
        0.7843
      ]
    ],
    [
      0.0938,
      [
        1.0,
        0.0278,
        0.1275,
        0.6383
      ]
    ],
    [
      0.2031,
      [
        1.0,
        0.0278,
        0.1275,
        0.4955
      ]
    ],
    [
      0.3125,
      [
        1.0,
        0.0278,
        0.1275,
        0.3785
      ]
    ],
    [
      0.4219,
      [
        1.0,
        0.0278,
        0.1275,
        0.2838
      ]
    ],
    [
      0.6562,
      [
        1.0,
        0.0278,
        0.1275,
        0.138
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0278,
        0.1275,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
