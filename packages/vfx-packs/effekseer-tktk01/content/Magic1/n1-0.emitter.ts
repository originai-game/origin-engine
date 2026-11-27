import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "破裂衝撃"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.25,
    0.4167
  ],
  "speed": [
    3.0,
    21.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1308
    ],
    [
      0.25,
      1.3814
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      0.5423
    ],
    [
      1.0,
      0.4657
    ]
  ],
  "size": [
    3.5,
    10.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5714
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
        0.3345,
        0.8832,
        0.949,
        0.0588
      ]
    ],
    [
      0.7,
      [
        0.3345,
        0.8832,
        0.949,
        0.0588
      ]
    ],
    [
      1.0,
      [
        0.3345,
        0.8832,
        0.949,
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
