import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.1.1.0 "LineGlow"
export default file.particles({
  "maxParticles": 250,
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
    4.7812,
    4.7812
  ],
  "heightOverLife": [
    [
      0.0,
      0.4167
    ],
    [
      1.0,
      0.4167
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
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.0478,
        0.0104,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.1255
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.5356,
        0.1166,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1166,
        0.1255
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
