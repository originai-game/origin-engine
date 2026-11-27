import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2.0.0.1.0 "particle_LineColor"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 6000.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    2.1429,
    11.5714
  ],
  "speedOverLife": [
    [
      0.0,
      5.2564
    ],
    [
      0.2031,
      2.1303
    ],
    [
      0.2969,
      1.1396
    ],
    [
      0.4062,
      0.3723
    ],
    [
      0.5312,
      0.0079
    ],
    [
      0.6875,
      0.3212
    ],
    [
      0.7188,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    2.0,
    3.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.0478,
        0.0478,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.0292,
        0.0292,
        0.0292,
        0.251
      ]
    ],
    [
      0.6,
      [
        0.0064,
        0.0064,
        0.0064,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.5356,
        0.5356,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.4591,
        0.4591,
        0.4591,
        0.251
      ]
    ],
    [
      0.6,
      [
        0.325,
        0.325,
        0.325,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
