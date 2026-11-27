import { file } from '@origingame/engine/author';
// Effekseer node 3.0.5.0.0.0 "particleMain"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.3125,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
  ],
  "speed": [
    0.8203,
    1.9922
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
    0.625,
    0.625
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
      0.1,
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
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
