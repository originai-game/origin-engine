import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.1.0.1 "LineColor"
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
        0.2195,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.2195,
        0.0478,
        1.0,
        0.502
      ]
    ],
    [
      0.375,
      [
        0.2195,
        0.0478,
        1.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0478,
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
