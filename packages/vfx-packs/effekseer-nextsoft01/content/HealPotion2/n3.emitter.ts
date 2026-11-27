import { file } from '@origingame/engine/author';
// Effekseer node 3 "粒子"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "point"
  },
  "spread": 11.9767,
  "rate": 300.0,
  "duration": 0.1333,
  "looping": false,
  "lifetime": [
    0.5,
    0.8333
  ],
  "speed": [
    12.267,
    16.9918
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
        0.2084,
        0.2084,
        1.0
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.2084,
        0.2084,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2084,
        0.2084,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -36.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Particle01"
  },
  "blend": "additive"
});
