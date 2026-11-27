import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.0 "炎"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.0167,
    0.8333
  ],
  "speed": [
    0.0,
    10.5882
  ],
  "size": [
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
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
        0.0104,
        0.0023,
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0104,
        0.0023,
        0.1961
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.0667,
        0.0145,
        0.144
      ]
    ],
    [
      0.5156,
      [
        1.0,
        0.1718,
        0.0374,
        0.095
      ]
    ],
    [
      0.7656,
      [
        1.0,
        0.3321,
        0.0723,
        0.046
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
