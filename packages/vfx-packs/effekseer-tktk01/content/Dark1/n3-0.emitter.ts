import { file } from '@origingame/engine/author';
// Effekseer node 3.0 "登る柱"
export default file.particles({
  "maxParticles": 4096,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.0,
      0.001
    ]
  },
  "spread": 5.8036,
  "rate": 12000.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.6667
  ],
  "speed": [
    44.2267,
    56.1783
  ],
  "speedOverLife": [
    [
      0.0,
      3.6895
    ],
    [
      0.25,
      1.5124
    ],
    [
      0.375,
      0.7867
    ],
    [
      0.5,
      0.3172
    ],
    [
      0.625,
      0.1037
    ],
    [
      0.75,
      0.1464
    ],
    [
      0.875,
      0.4452
    ],
    [
      1.0,
      0.9788
    ]
  ],
  "size": [
    2.0,
    2.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      0.0781,
      0.6313
    ],
    [
      0.1719,
      0.7535
    ],
    [
      0.2656,
      0.8423
    ],
    [
      0.375,
      0.9111
    ],
    [
      0.5625,
      0.9653
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
        0.5989,
        0.043,
        0.7372,
        0.1961
      ]
    ],
    [
      0.5,
      [
        0.5989,
        0.043,
        0.7372,
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.5989,
        0.043,
        0.7372,
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
