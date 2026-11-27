import { file } from '@origingame/engine/author';
// Effekseer node 5.1.0.0.0.0 "particleMain"
export default file.particles({
  "maxParticles": 188,
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
  "duration": 0.0208,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    2.25,
    6.75
  ],
  "speedOverLife": [
    [
      0.0,
      2.9785
    ],
    [
      0.1562,
      2.1316
    ],
    [
      0.2969,
      1.4897
    ],
    [
      0.5,
      0.7996
    ],
    [
      0.7031,
      0.3896
    ],
    [
      0.7188,
      0.0809
    ],
    [
      0.7344,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.25,
    1.25
  ],
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
      0.2857,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.4286,
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
