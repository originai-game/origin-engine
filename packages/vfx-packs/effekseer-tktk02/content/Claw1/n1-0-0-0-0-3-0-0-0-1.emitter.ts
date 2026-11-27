import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.0.3.0.0.0.1 "particleColor"
export default file.particles({
  "maxParticles": 377,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.6667,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.8,
    4.0
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
    1.3333,
    1.3333
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.0478,
        0.0104,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2195,
        0.0478,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
