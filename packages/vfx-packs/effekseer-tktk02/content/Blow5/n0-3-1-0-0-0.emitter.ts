import { file } from '@origingame/engine/author';
// Effekseer node 0.3.1.0.0.0 "LineMain"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    3.75,
    9.5833
  ],
  "speedOverLife": [
    [
      0.0,
      2.4854
    ],
    [
      0.2344,
      1.6165
    ],
    [
      0.4531,
      0.9849
    ],
    [
      0.6406,
      0.5931
    ],
    [
      0.8438,
      0.3246
    ],
    [
      0.875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.8333,
    0.8333
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
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line6"
  },
  "blend": "additive"
});
