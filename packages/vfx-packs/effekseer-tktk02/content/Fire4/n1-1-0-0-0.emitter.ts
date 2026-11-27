import { file } from '@origingame/engine/author';
// Effekseer node 1.1.0.0.0 "FineMain"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.0625,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    1.625,
    3.0875
  ],
  "speedOverLife": [
    [
      0.0,
      1.4238
    ],
    [
      0.5,
      0.9679
    ],
    [
      1.0,
      0.7016
    ]
  ],
  "size": [
    0.4333,
    0.4333
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
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
      0.25,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.251
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
