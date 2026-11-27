import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.95,
    4.65
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
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0104,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.147,
        0.007,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0023,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.4134,
        0.032,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
