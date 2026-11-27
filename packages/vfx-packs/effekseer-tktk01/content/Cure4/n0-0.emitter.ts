import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "メイン光"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.75,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 240.0,
  "duration": 0.2083,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.5,
    6.5
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
    3.75,
    6.25
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.1562,
      0.2599
    ],
    [
      0.3281,
      0.3758
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        0.3112,
        0.3922
      ]
    ],
    [
      0.2656,
      [
        1.0,
        1.0,
        0.5651,
        0.3922
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        0.7332,
        0.3922
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
