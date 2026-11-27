import { file } from '@origingame/engine/author';
// Effekseer node 1.0.2.0 "追尾粒子"
export default file.particles({
  "maxParticles": 3813,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 12000.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.1667,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.0,
    1.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0478,
        0.3922
      ]
    ],
    [
      0.4844,
      [
        1.0,
        0.0111,
        0.0111,
        0.2022
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
