import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0 "Gloｗ"
export default file.particles({
  "maxParticles": 250,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.5,
    3.75
  ],
  "speedOverLife": [
    [
      0.0,
      4.2345
    ],
    [
      0.125,
      2.763
    ],
    [
      0.1875,
      2.1717
    ],
    [
      0.25,
      1.6949
    ],
    [
      0.375,
      1.0848
    ],
    [
      0.4844,
      0.9269
    ],
    [
      0.5156,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        0.0,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.3922
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.3922
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
