import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "粒ﾉｰﾄﾞ"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.0,
      0.001
    ]
  },
  "spread": 90.0,
  "rate": 300.0,
  "duration": 0.1333,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    4.2426,
    9.9499
  ],
  "size": [
    0.5,
    0.5
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
        0.7047,
        1.0,
        1.0,
        0.5882
      ]
    ],
    [
      1.0,
      [
        0.7047,
        1.0,
        1.0,
        0.5882
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
