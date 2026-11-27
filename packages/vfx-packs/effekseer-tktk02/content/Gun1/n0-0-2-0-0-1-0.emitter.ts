import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2.0.0.1.0 "particleGlow"
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
  "rate": 3000.0,
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.6,
    9.0
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
      0.0808
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
      0.2857,
      [
        0.0228,
        0.0228,
        0.0228,
        0.251
      ]
    ],
    [
      0.4286,
      [
        0.0139,
        0.0139,
        0.0139,
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
      0.2857,
      [
        0.4282,
        0.4282,
        0.4282,
        0.251
      ]
    ],
    [
      0.4286,
      [
        0.3794,
        0.3794,
        0.3794,
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
