import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.0.0.0 "粒子色付け"
export default file.particles({
  "maxParticles": 625,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.4,
      0.1333,
      0.1333
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.2083,
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
    0.6667,
    0.6667
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
