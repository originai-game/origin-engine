import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "火の玉中心"
export default file.particles({
  "maxParticles": 252,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.5833,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    4.5,
    4.5
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
      0.0781,
      0.7405
    ],
    [
      0.1562,
      0.5398
    ],
    [
      0.2344,
      0.3903
    ],
    [
      0.3281,
      0.2672
    ],
    [
      0.4688,
      0.1698
    ],
    [
      0.8281,
      0.105
    ],
    [
      1.0,
      0.0
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
        0.8046,
        0.6388,
        0.1247,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.9743,
        0.6388,
        0.142,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.8046,
        0.8513,
        0.1247,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.9743,
        0.6388,
        0.142,
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
