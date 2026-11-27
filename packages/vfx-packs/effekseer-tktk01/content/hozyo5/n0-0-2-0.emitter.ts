import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2.0 "追尾粒子"
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
        0.0478,
        0.0478,
        1.0,
        0.3922
      ]
    ],
    [
      0.4844,
      [
        0.0111,
        0.0111,
        1.0,
        0.2022
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.2195,
        1.0,
        0.3922
      ]
    ],
    [
      0.2344,
      [
        0.0265,
        0.122,
        1.0,
        0.3002
      ]
    ],
    [
      0.4844,
      [
        0.0111,
        0.0511,
        1.0,
        0.2022
      ]
    ],
    [
      0.7344,
      [
        0.0026,
        0.0119,
        1.0,
        0.1042
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
