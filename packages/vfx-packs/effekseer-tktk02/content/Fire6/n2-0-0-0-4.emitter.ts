import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.4 "Glow1"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.25,
    1.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.0233,
        0.0104,
        0.251
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0175,
        0.0104,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.0104,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.1562,
      [
        1.0,
        0.1538,
        0.0335,
        0.1176
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.107,
        0.0233,
        0.251
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0802,
        0.0175,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
