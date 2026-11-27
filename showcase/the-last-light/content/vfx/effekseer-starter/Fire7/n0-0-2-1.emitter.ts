import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "facing",
    "normal": [
      0,
      0,
      1
    ]
  },
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        0,
        0
      ]
    ],
    [
      0.33329999446868896,
      [
        1,
        1,
        0,
        0.25099998712539673
      ]
    ],
    [
      1,
      [
        1,
        1,
        0,
        0
      ]
    ]
  ],
  "distortion": 0.10000000149011612,
  "duration": 0.16670000553131104,
  "lifetime": [
    0.33329999446868896,
    0.666700005531311
  ],
  "looping": false,
  "maxParticles": 1250,
  "rate": 600,
  "rotation": [
    -180,
    180
  ],
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ]
    ]
  },
  "size": [
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0,
      0.10000000149011612
    ],
    [
      0.3125,
      0.48240000009536743
    ],
    [
      0.5625,
      0.7389000058174133
    ],
    [
      0.7968999743461609,
      0.9147999882698059
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    0.5,
    3.5
  ],
  "speedOverLife": [
    [
      0,
      1.423799991607666
    ],
    [
      0.5,
      0.9678999781608582
    ],
    [
      1,
      0.7016000151634216
    ]
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Distortion11")
});
