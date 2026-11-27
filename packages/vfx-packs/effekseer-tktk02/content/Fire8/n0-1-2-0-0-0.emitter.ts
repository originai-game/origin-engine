import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2.0.0.0 "particleMain"
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
  "rate": 600.0,
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.5,
    4.0
  ],
  "speedOverLife": [
    [
      0.0,
      1.8997
    ],
    [
      0.3594,
      1.395
    ],
    [
      0.7344,
      0.6571
    ],
    [
      0.7656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.5,
    0.5
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
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.502
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
