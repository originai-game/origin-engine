import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "花びら"
export default file.particles({
  "maxParticles": 188,
  "shape": {
    "kind": "point"
  },
  "spread": 25.1888,
  "rate": 6000.0,
  "duration": 0.025,
  "looping": false,
  "lifetime": [
    0.25,
    0.4833
  ],
  "speed": [
    2.7437,
    8.447
  ],
  "speedOverLife": [
    [
      0.0,
      3.6098
    ],
    [
      0.1406,
      2.3125
    ],
    [
      0.25,
      1.4926
    ],
    [
      0.375,
      0.7978
    ],
    [
      0.4844,
      0.4136
    ],
    [
      0.6094,
      0.2431
    ],
    [
      0.7812,
      0.2885
    ],
    [
      1.0,
      0.9798
    ]
  ],
  "size": [
    1.0,
    1.0
  ],
  "heightOverLife": [
    [
      0.0,
      0.4
    ],
    [
      1.0,
      0.4
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
        0.9574,
        0.0278,
        0.1247,
        1.0
      ]
    ],
    [
      0.5455,
      [
        0.9574,
        0.0278,
        0.1247,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.9574,
        0.0278,
        0.1247,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.9574,
        0.3112,
        0.5795,
        1.0
      ]
    ],
    [
      0.5455,
      [
        0.9574,
        0.3112,
        0.5795,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.9574,
        0.3112,
        0.5795,
        0.0
      ]
    ]
  ],
  "angularVelocity": [
    -1963.6364,
    1963.6364
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
