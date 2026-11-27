import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.0 "glow"
export default file.particles({
  "maxParticles": 938,
  "shape": {
    "kind": "box",
    "halfExtents": [
      4.0,
      0.75,
      0.001
    ]
  },
  "spread": 180.0,
  "rate": 6000.0,
  "duration": 0.125,
  "looping": false,
  "lifetime": [
    1.3333,
    1.3333
  ],
  "speed": [
    0.45,
    2.85
  ],
  "speedOverLife": [
    [
      0.0,
      6.7232
    ],
    [
      0.0781,
      4.4216
    ],
    [
      0.125,
      3.3082
    ],
    [
      0.2031,
      2.039
    ],
    [
      0.25,
      1.6292
    ],
    [
      0.2969,
      1.4834
    ],
    [
      0.3281,
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
        1.0,
        0.0478,
        0.0104,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        1.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.1166,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.1166,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
