import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.1.0 "LineMain"
export default file.particles({
  "maxParticles": 500,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 0.0833,
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
    0.75,
    0.75
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
        0.2195,
        0.2195,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.2195,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.2195,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
