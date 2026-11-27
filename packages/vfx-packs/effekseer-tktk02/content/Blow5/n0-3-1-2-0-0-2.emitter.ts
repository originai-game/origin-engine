import { file } from '@origingame/engine/author';
// Effekseer node 0.3.1.2.0.0.2 "LineGlow"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.25,
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
    1.4062,
    2.7188
  ],
  "speedOverLife": [
    [
      0.0,
      4.6075
    ],
    [
      0.2188,
      1.9913
    ],
    [
      0.3281,
      1.0748
    ],
    [
      0.4375,
      0.4387
    ],
    [
      0.5938,
      0.0164
    ],
    [
      0.7812,
      0.265
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
    1.25,
    1.25
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0104,
        0.0023,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.0023,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
