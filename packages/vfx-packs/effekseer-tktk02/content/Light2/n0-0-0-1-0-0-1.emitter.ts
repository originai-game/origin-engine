import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 875,
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
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.5,
    4.5
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
    0.875,
    1.125
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.7445,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.6707,
        1.0,
        0.007,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.6011,
        1.0,
        0.0043,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
        0.0023,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.8671,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.8251,
        1.0,
        0.032,
        0.502
      ]
    ],
    [
      0.6667,
      [
        0.7843,
        1.0,
        0.0196,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.7445,
        1.0,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
