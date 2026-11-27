import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2.1 "Distortion"
export default file.particles({
  "maxParticles": 1250,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ],
      [
        0.0,
        0.0,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    0.5,
    3.5
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
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1
    ],
    [
      0.3125,
      0.4824
    ],
    [
      0.5625,
      0.7389
    ],
    [
      0.7969,
      0.9148
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
        1.0,
        0.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        0.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Distortion11"
  },
  "blend": "additive",
  "distortion": 0.1
});
