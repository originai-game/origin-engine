import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.0.0.0.2.0.3.0.0.0.0 "particleMain"
export default file.particles({
  "maxParticles": 379,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5833,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 720.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.4167,
    0.4167
  ],
  "speed": [
    0.875,
    4.375
  ],
  "speedOverLife": [
    [
      0.0,
      2.6617
    ],
    [
      0.2031,
      1.7921
    ],
    [
      0.3906,
      1.1489
    ],
    [
      0.5938,
      0.6438
    ],
    [
      0.7812,
      0.3546
    ],
    [
      0.7969,
      0.2384
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
    0.5833,
    0.5833
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
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
