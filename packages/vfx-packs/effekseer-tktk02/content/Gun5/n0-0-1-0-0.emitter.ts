import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.0.0 "LineMain"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    1.125,
    2.625
  ],
  "speedOverLife": [
    [
      0.0,
      2.8378
    ],
    [
      0.1719,
      1.9946
    ],
    [
      0.3125,
      1.4159
    ],
    [
      0.5156,
      0.7848
    ],
    [
      0.7344,
      0.3758
    ],
    [
      0.7656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
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
        1.0
      ]
    ],
    [
      0.5,
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
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Line01"
  },
  "blend": "additive"
});
