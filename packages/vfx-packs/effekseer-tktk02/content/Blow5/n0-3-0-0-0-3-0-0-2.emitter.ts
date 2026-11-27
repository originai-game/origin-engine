import { file } from '@origingame/engine/author';
// Effekseer node 0.3.0.0.0.3.0.0.2 "LineMain"
export default file.particles({
  "maxParticles": 188,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.25,
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
    3.75,
    3.75
  ],
  "heightOverLife": [
    [
      0.0,
      0.3
    ],
    [
      1.0,
      0.3
    ]
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
        0.0,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        1.0,
        0.1255
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line6"
  },
  "blend": "additive"
});
