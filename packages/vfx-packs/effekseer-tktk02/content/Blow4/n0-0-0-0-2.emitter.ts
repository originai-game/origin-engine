import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.2 "ParticleGlow"
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
  "rate": 2400.0,
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
      4.4254
    ],
    [
      0.2188,
      1.9936
    ],
    [
      0.3281,
      1.1232
    ],
    [
      0.4375,
      0.5008
    ],
    [
      0.6094,
      0.024
    ],
    [
      0.8125,
      0.2502
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
    2.5,
    2.5
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
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
