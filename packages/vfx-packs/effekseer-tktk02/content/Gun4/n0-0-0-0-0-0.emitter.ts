import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.0 "LineMain"
export default file.particles({
  "maxParticles": 3250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    1.875,
    5.625
  ],
  "speedOverLife": [
    [
      0.0,
      2.6617
    ],
    [
      0.2031,
      1.7921
    ],
    [
      0.3906,
      1.1489
    ],
    [
      0.5938,
      0.6438
    ],
    [
      0.7812,
      0.3546
    ],
    [
      0.7969,
      0.2384
    ],
    [
      0.8125,
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
      0.8,
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
