import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.0 "particleMain"
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
    0.25,
    0.25
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
