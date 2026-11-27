import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.1 "particleGlow"
export default file.particles({
  "maxParticles": 377,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      2.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 1.0,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.2679,
    2.0893
  ],
  "speedOverLife": [
    [
      0.0,
      1.9174
    ],
    [
      0.5,
      0.9319
    ],
    [
      0.75,
      0.5877
    ],
    [
      1.0,
      0.3485
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
        0.5356,
        1.0,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.4723,
        1.0,
        0.007,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.4423,
        1.0,
        0.0055,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.3587,
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
        0.7445,
        1.0,
        0.0478,
        0.0
      ]
    ],
    [
      0.3333,
      [
        0.6707,
        1.0,
        0.032,
        0.251
      ]
    ],
    [
      0.5,
      [
        0.6354,
        1.0,
        0.0254,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.5356,
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
