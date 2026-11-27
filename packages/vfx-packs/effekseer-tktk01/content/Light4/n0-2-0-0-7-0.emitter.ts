import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.7.0 "粒子"
export default file.particles({
  "maxParticles": 375,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 7200.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.75,
    0.75
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.2812,
      0.3346
    ],
    [
      0.5938,
      0.6602
    ],
    [
      0.6094,
      0.651
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.04,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.6,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
  "angularVelocity": [
    -1200.0,
    1200.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
