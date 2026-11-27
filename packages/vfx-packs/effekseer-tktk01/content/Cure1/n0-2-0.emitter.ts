import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0 "光追走"
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
        0.9075,
        0.9914,
        0.212,
        0.2941
      ]
    ],
    [
      0.25,
      [
        0.9302,
        0.9935,
        0.3501,
        0.2206
      ]
    ],
    [
      0.5,
      [
        0.9532,
        0.9957,
        0.5265,
        0.1471
      ]
    ],
    [
      0.75,
      [
        0.9764,
        0.9978,
        0.7427,
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
