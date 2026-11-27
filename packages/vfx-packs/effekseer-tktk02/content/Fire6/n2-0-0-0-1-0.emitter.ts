import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.0.1.0 "particleColor"
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
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.75,
    2.25
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
      0.25,
      [
        1.0,
        0.0356,
        0.0078,
        0.251
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0254,
        0.0055,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0104,
        0.0023,
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
      0.25,
      [
        1.0,
        0.1636,
        0.0356,
        0.251
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0254,
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
