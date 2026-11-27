import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.1.0.2 "LineGlow"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
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
        0.0478,
        0.0104,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.0478,
        0.0104,
        1.0,
        0.1255
      ]
    ],
    [
      0.375,
      [
        0.0478,
        0.0104,
        1.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
