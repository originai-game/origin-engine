import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.3.0.0.0.2 "particleGlow"
export default file.particles({
  "maxParticles": 152,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5833,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 240.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.05,
    3.15
  ],
  "speedOverLife": [
    [
      0.0,
      2.5559
    ],
    [
      0.2344,
      1.64
    ],
    [
      0.4531,
      0.9806
    ],
    [
      0.625,
      0.6061
    ],
    [
      0.8125,
      0.3417
    ],
    [
      0.8281,
      0.273
    ],
    [
      0.8438,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    5.8333,
    5.8333
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.5356,
        0.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.2195,
        0.5356,
        0.0,
        0.1255
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.5356,
        0.0,
        0.1255
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.7445,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.5356,
        0.7445,
        0.0104,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.7445,
        0.0104,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
