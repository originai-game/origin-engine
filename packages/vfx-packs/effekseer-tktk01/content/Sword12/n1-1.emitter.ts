import { file } from '@origingame/engine/author';
// Effekseer node 1.1 "粒ﾉｰﾄﾞ余韻"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.0,
      0.001
    ]
  },
  "spread": 90.0,
  "rate": 300.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    2.8284,
    6.6332
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
    0.25,
    0.25
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.7047,
        1.0,
        1.0,
        0.5882
      ]
    ],
    [
      1.0,
      [
        0.7047,
        1.0,
        1.0,
        0.5882
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
