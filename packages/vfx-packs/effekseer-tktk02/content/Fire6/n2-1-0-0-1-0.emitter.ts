import { file } from '@origingame/engine/author';
// Effekseer node 2.1.0.0.1.0 "particleColor"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.5,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 2400.0,
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    5.0,
    15.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.8378
    ],
    [
      0.1719,
      1.9946
    ],
    [
      0.3125,
      1.4159
    ],
    [
      0.5156,
      0.7848
    ],
    [
      0.7344,
      0.3758
    ],
    [
      0.7656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    4.375,
    5.625
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
