import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "光追走"
export default file.particles({
  "maxParticles": 757,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 240.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.25,
    0.75
  ],
  "speed": [
    1.4614,
    5.8457
  ],
  "size": [
    0.5625,
    0.5625
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
        0.8434,
        0.8913,
        0.139,
        0.2941
      ]
    ],
    [
      0.25,
      [
        0.8812,
        0.9178,
        0.2748,
        0.2206
      ]
    ],
    [
      0.5,
      [
        0.9199,
        0.9448,
        0.4619,
        0.1471
      ]
    ],
    [
      0.75,
      [
        0.9595,
        0.9722,
        0.7029,
        0.0735
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
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
