import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.1.1 "LineColor"
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
    1.5,
    1.5
  ],
  "heightOverLife": [
    [
      0.0,
      0.625
    ],
    [
      1.0,
      0.625
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
        0.0104,
        0.0023,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.0104,
        0.0023,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.0023,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.1166,
        0.0254,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.1166,
        0.0254,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
