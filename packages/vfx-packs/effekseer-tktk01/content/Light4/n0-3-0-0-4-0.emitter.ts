import { file } from '@origingame/engine/author';
// Effekseer node 0.3.0.0.4.0 "粒子"
export default file.particles({
  "maxParticles": 255,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 720.0,
  "duration": 0.2833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.25,
    0.75
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0333,
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
        1.0
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
