import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.1.0 "particleColor"
export default file.particles({
  "maxParticles": 750,
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
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    3.75,
    11.25
  ],
  "speedOverLife": [
    [
      0.0,
      4.6075
    ],
    [
      0.2188,
      1.9913
    ],
    [
      0.3281,
      1.0748
    ],
    [
      0.4375,
      0.4387
    ],
    [
      0.5938,
      0.0164
    ],
    [
      0.7812,
      0.265
    ],
    [
      0.8125,
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
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.4,
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
