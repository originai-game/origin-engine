import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0.0.2.0.0.0 "particleMain"
export default file.particles({
  "maxParticles": 188,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    0.7339,
    1.3982
  ],
  "speedOverLife": [
    [
      0.0,
      5.8021
    ],
    [
      0.1875,
      2.2912
    ],
    [
      0.2812,
      1.1208
    ],
    [
      0.375,
      0.3714
    ],
    [
      0.4844,
      0.029
    ],
    [
      0.6562,
      0.2173
    ],
    [
      0.6875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.25,
    0.25
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
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.1255
      ]
    ],
    [
      0.6667,
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
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle01"
  },
  "blend": "additive"
});
