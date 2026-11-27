import { file } from '@origingame/engine/author';
// Effekseer node 0.1.3.0 "粒子"
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
    0.3333,
    0.6667
  ],
  "speed": [
    16.2,
    19.8
  ],
  "size": [
    0.4,
    0.6
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0781,
      0.7375
    ],
    [
      0.1719,
      0.493
    ],
    [
      0.2656,
      0.3153
    ],
    [
      0.375,
      0.1779
    ],
    [
      0.5625,
      0.0695
    ],
    [
      0.875,
      0.0366
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
  "gravity": [
    0.0,
    0.0,
    -28.8
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
