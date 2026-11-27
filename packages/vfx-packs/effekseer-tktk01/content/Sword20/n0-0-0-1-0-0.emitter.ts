import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1.0.0 "粒子色付け"
export default file.particles({
  "maxParticles": 2800,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 13440.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    52.5,
    52.5
  ],
  "size": [
    0.35,
    0.35
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
