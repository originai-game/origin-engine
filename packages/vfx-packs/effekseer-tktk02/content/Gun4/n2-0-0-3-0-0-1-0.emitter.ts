import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.3.0.0.1.0 "particleGlow"
export default file.particles({
  "maxParticles": 938,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 1200.0,
  "duration": 0.125,
  "looping": false,
  "lifetime": [
    1.0833,
    1.0833
  ],
  "speed": [
    0.9,
    5.1
  ],
  "speedOverLife": [
    [
      0.0,
      4.7895
    ],
    [
      0.2188,
      1.984
    ],
    [
      0.3125,
      1.141
    ],
    [
      0.4219,
      0.4504
    ],
    [
      0.5781,
      0.011
    ],
    [
      0.75,
      0.271
    ],
    [
      0.7812,
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
      0.1538,
      [
        0.0331,
        0.0331,
        0.0331,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.251
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
      0.1538,
      [
        0.4762,
        0.4762,
        0.4762,
        0.251
      ]
    ],
    [
      0.5781,
      [
        0.3317,
        0.3317,
        0.3317,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
