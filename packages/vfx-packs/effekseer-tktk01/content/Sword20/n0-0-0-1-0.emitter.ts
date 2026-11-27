import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1.0 "粒子"
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
    300.0,
    300.0
  ],
  "size": [
    0.15,
    0.2
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1094,
      0.7835
    ],
    [
      0.2188,
      0.6011
    ],
    [
      0.3281,
      0.4494
    ],
    [
      0.4531,
      0.3096
    ],
    [
      0.5781,
      0.2009
    ],
    [
      0.7031,
      0.1185
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
