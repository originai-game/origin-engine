import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2.0.0.1 "particle_LineColor"
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
    1.0714,
    5.7857
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
    0.875,
    1.125
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0,
        0.0,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.1344,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      0.3906,
      [
        0.0738,
        0.0,
        0.0,
        1.0
      ]
    ],
    [
      0.6,
      [
        0.0292,
        0.0,
        0.0,
        1.0
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
        0.5295,
        0.2158,
        0.2158,
        0.0
      ]
    ],
    [
      0.2,
      [
        0.4546,
        0.2165,
        0.2165,
        1.0
      ]
    ],
    [
      0.6,
      [
        0.3232,
        0.218,
        0.218,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2195,
        0.2195,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
