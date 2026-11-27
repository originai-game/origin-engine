import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.1.0 "粒子"
export default file.particles({
  "maxParticles": 94,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.125,
  "looping": false,
  "lifetime": [
    0.25,
    0.4167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.5417,
    0.5417
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.0
    ]
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
      0.05,
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
