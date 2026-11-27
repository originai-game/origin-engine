import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0.2.2 "particleGlow"
export default file.particles({
  "maxParticles": 750,
  "shape": {
    "kind": "sphere",
    "radius": 0.875
  },
  "spread": 0.0,
  "rate": 1800.0,
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.6667,
    0.6667
  ],
  "speed": [
    0.2188,
    0.3646
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
    1.1667,
    1.75
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0478,
        0.0104,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.0356,
        0.0078,
        1.0,
        0.251
      ]
    ],
    [
      0.75,
      [
        0.017,
        0.0037,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0104,
        0.0023,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.1636,
        0.0356,
        1.0,
        0.251
      ]
    ],
    [
      0.75,
      [
        0.0781,
        0.017,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0478,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
