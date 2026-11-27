import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0.0.0.3.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 438,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 4200.0,
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.45,
    1.35
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
    0.4375,
    0.5625
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.2195,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2195,
        0.2195,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.2195,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.5356,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.5356,
        0.5356,
        0.7529
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.5356,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
