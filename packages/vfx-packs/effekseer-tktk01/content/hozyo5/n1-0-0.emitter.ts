import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "glow"
export default file.particles({
  "maxParticles": 52,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.299,
      0.001
    ]
  },
  "spread": 29.4962,
  "rate": 120.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    1.9486,
    5.8457
  ],
  "speedOverLife": [
    [
      0.0,
      0.4657
    ],
    [
      0.25,
      0.5423
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      1.3814
    ],
    [
      1.0,
      2.1308
    ]
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0104,
        1.0,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0104,
        1.0,
        0.3922
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0,
        0.1166,
        1.0,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.1166,
        1.0,
        0.3922
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
